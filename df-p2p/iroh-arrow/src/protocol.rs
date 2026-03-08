use std::sync::Arc;

use arrow::array::RecordBatch;
use iroh::{
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler},
};
use tracing::info;

/// Default ALPN protocol identifier for iroh-arrow exchanges.
pub const ALPN: &[u8] = b"mark/iroh-arrow/0";

/// Default maximum payload size for receiving Arrow data (1 GB).
pub const MAX_RESPONSE_SIZE: usize = 1024 * 1024 * 1024;

/// Default maximum request size (64 KB).
pub const MAX_REQUEST_SIZE: usize = 64 * 1024;

/// Trait for handling incoming requests on the Arrow protocol.
///
/// Implementors receive raw request bytes and return Arrow record batches.
/// For example, a DataFusion handler would parse the bytes as a SQL string,
/// execute the query, and return the resulting batches.
pub trait RequestHandler: Send + Sync + std::fmt::Debug + 'static {
    fn handle(
        &self,
        request: &[u8],
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<RecordBatch>>> + Send;
}

/// Protocol handler that receives requests over iroh QUIC and responds with Arrow IPC data.
///
/// Wire protocol (over iroh QUIC bidirectional stream):
/// - **Request**: raw bytes from client, then client finishes send
/// - **Response**: Arrow IPC stream bytes (schema + batches), then server finishes send
///
/// Register on an iroh `Router` with [`ALPN`]:
/// ```ignore
/// let handler = IrohArrow::new(my_handler);
/// let router = Router::builder(endpoint)
///     .accept(ALPN, handler)
///     .spawn();
/// ```
#[derive(Debug, Clone)]
pub struct IrohArrow<H> {
    handler: Arc<H>,
    max_request_size: usize,
}

impl<H: RequestHandler> IrohArrow<H> {
    pub fn new(handler: H) -> Self {
        Self {
            handler: Arc::new(handler),
            max_request_size: MAX_REQUEST_SIZE,
        }
    }

    pub fn with_max_request_size(mut self, size: usize) -> Self {
        self.max_request_size = size;
        self
    }
}

fn accept_err(e: anyhow::Error) -> AcceptError {
    AcceptError::from_boxed(e.into())
}

impl<H: RequestHandler> ProtocolHandler for IrohArrow<H> {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let (mut send, mut recv) = connection.accept_bi().await?;

        // Read request bytes (client finishes its send side after writing)
        let request_bytes = crate::stream::recv_bytes(&mut recv, self.max_request_size)
            .await
            .map_err(accept_err)?;
        info!(request_len = request_bytes.len(), "received request");

        // Delegate to the user-provided handler
        let batches = self
            .handler
            .handle(&request_bytes)
            .await
            .map_err(accept_err)?;
        info!(num_batches = batches.len(), "sending response");

        // Send results as Arrow IPC
        crate::stream::send_batches(&mut send, &batches)
            .await
            .map_err(accept_err)?;

        // Keep connection alive until the client has read all data.
        // Without this, dropping `conn` sends an immediate CONNECTION_CLOSE
        // that can abort in-flight stream data.
        connection.closed().await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use iroh::protocol::Router;

    /// Echo handler — interprets request bytes as a UTF-8 table name
    /// and returns a fixed batch with that name embedded.
    #[derive(Debug, Clone)]
    struct EchoHandler;

    impl RequestHandler for EchoHandler {
        async fn handle(&self, request: &[u8]) -> anyhow::Result<Vec<RecordBatch>> {
            let msg = std::str::from_utf8(request)?;
            let schema = Arc::new(Schema::new(vec![
                Field::new("request", DataType::Utf8, false),
                Field::new("value", DataType::Int32, false),
            ]));
            let batch = RecordBatch::try_new(
                schema,
                vec![
                    Arc::new(StringArray::from(vec![msg])),
                    Arc::new(Int32Array::from(vec![42])),
                ],
            )?;
            Ok(vec![batch])
        }
    }

    /// Handler that returns no batches.
    #[derive(Debug, Clone)]
    struct EmptyHandler;

    impl RequestHandler for EmptyHandler {
        async fn handle(&self, _request: &[u8]) -> anyhow::Result<Vec<RecordBatch>> {
            Ok(vec![])
        }
    }

    /// Handler that returns multiple batches to test multi-batch streaming.
    #[derive(Debug, Clone)]
    struct MultiBatchHandler;

    impl RequestHandler for MultiBatchHandler {
        async fn handle(&self, _request: &[u8]) -> anyhow::Result<Vec<RecordBatch>> {
            let schema = Arc::new(Schema::new(vec![
                Field::new("n", DataType::Int32, false),
            ]));
            let b1 = RecordBatch::try_new(
                Arc::clone(&schema),
                vec![Arc::new(Int32Array::from(vec![1, 2, 3]))],
            )?;
            let b2 = RecordBatch::try_new(
                Arc::clone(&schema),
                vec![Arc::new(Int32Array::from(vec![4, 5]))],
            )?;
            Ok(vec![b1, b2])
        }
    }

    /// Handler that returns an error.
    #[derive(Debug, Clone)]
    struct FailHandler;

    impl RequestHandler for FailHandler {
        async fn handle(&self, _request: &[u8]) -> anyhow::Result<Vec<RecordBatch>> {
            anyhow::bail!("intentional failure")
        }
    }

    /// Helper: spin up a server with the given handler and return (server_addr, router).
    async fn start_server<H: RequestHandler + Clone>(handler: H) -> (iroh::EndpointAddr, Router) {
        let ep = iroh::Endpoint::builder().bind().await.unwrap();
        ep.online().await;
        let addr = ep.addr();
        let router = Router::builder(ep)
            .accept(ALPN, IrohArrow::new(handler))
            .spawn();
        (addr, router)
    }

    /// Helper: connect a client, send request bytes, receive response batches.
    async fn request(addr: iroh::EndpointAddr, payload: &[u8]) -> Vec<RecordBatch> {
        let ep = iroh::Endpoint::builder().bind().await.unwrap();
        let conn = ep.connect(addr, ALPN).await.unwrap();
        let (mut send, mut recv) = conn.open_bi().await.unwrap();
        crate::stream::send_bytes(&mut send, payload).await.unwrap();
        crate::stream::recv_batches(&mut recv, MAX_RESPONSE_SIZE)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn end_to_end_echo() {
        let (addr, router) = start_server(EchoHandler).await;
        let batches = request(addr, b"hello").await;

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].num_rows(), 1);

        let col = batches[0]
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(col.value(0), "hello");

        let val = batches[0]
            .column(1)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();
        assert_eq!(val.value(0), 42);

        router.shutdown().await.ok();
    }

    #[tokio::test]
    async fn end_to_end_empty_response() {
        let (addr, router) = start_server(EmptyHandler).await;
        let batches = request(addr, b"ignored").await;
        assert!(batches.is_empty());
        router.shutdown().await.ok();
    }

    #[tokio::test]
    async fn end_to_end_multi_batch() {
        let (addr, router) = start_server(MultiBatchHandler).await;
        let batches = request(addr, b"go").await;

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].num_rows(), 3);
        assert_eq!(batches[1].num_rows(), 2);

        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 5);

        router.shutdown().await.ok();
    }

    #[tokio::test]
    async fn end_to_end_empty_request() {
        // EchoHandler will interpret empty bytes as empty UTF-8 string
        let (addr, router) = start_server(EchoHandler).await;
        let batches = request(addr, b"").await;

        assert_eq!(batches.len(), 1);
        let col = batches[0]
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(col.value(0), "");

        router.shutdown().await.ok();
    }

    #[tokio::test]
    async fn handler_error_closes_connection() {
        let (addr, router) = start_server(FailHandler).await;

        let ep = iroh::Endpoint::builder().bind().await.unwrap();
        let conn = ep.connect(addr, ALPN).await.unwrap();
        let (mut send, mut recv) = conn.open_bi().await.unwrap();
        crate::stream::send_bytes(&mut send, b"trigger error")
            .await
            .unwrap();

        // Should fail to receive because the handler errored
        let result = crate::stream::recv_batches(&mut recv, MAX_RESPONSE_SIZE).await;
        assert!(result.is_err());

        router.shutdown().await.ok();
    }

    #[tokio::test]
    async fn with_max_request_size_config() {
        let handler = IrohArrow::new(EchoHandler);
        assert_eq!(handler.max_request_size, MAX_REQUEST_SIZE);

        let handler = handler.with_max_request_size(1024 * 1024);
        assert_eq!(handler.max_request_size, 1024 * 1024);
    }
}
