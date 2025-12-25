use std::pin::Pin;
use std::usize;

use anyhow::Result;
use arrow::array::RecordBatch;
use arrow_flight::{
    Action, ActionType, Criteria, Empty, FlightData, FlightDescriptor, FlightInfo,
    HandshakeRequest, HandshakeResponse, PollInfo, PutResult, SchemaResult, Ticket,
    decode::FlightRecordBatchStream,
    encode::FlightDataEncoderBuilder,
    error::FlightError,
    flight_service_server::{FlightService, FlightServiceServer},
};
use dashmap::DashMap;
use datafusion_partition_exp::flight::{GetCommand, PutCommand};
use futures::{
    Stream, StreamExt, TryStreamExt,
    stream::{BoxStream, Peekable},
};
use prost::Message;
use tonic::{Request, Response, Status, Streaming, transport::Server};

#[derive(Clone)]
pub struct FlightServiceImpl {
    data: DashMap<usize, Vec<RecordBatch>>,
}

impl FlightServiceImpl {
    pub fn new() -> Self {
        Self {
            data: DashMap::new(),
        }
    }
}

pub struct PeekableStream {
    inner: Peekable<Streaming<FlightData>>,
}

impl PeekableStream {
    fn new(stream: Streaming<FlightData>) -> Self {
        Self {
            inner: stream.peekable(),
        }
    }

    pub async fn peek(&mut self) -> Option<&Result<FlightData, Status>> {
        Pin::new(&mut self.inner).peek().await
    }
}

impl Stream for PeekableStream {
    type Item = Result<FlightData, Status>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx)
    }
}

#[tonic::async_trait]
impl FlightService for FlightServiceImpl {
    type HandshakeStream = BoxStream<'static, Result<HandshakeResponse, Status>>;
    type ListFlightsStream = BoxStream<'static, Result<FlightInfo, Status>>;
    type DoGetStream = BoxStream<'static, Result<FlightData, Status>>;
    type DoPutStream = BoxStream<'static, Result<PutResult, Status>>;
    type DoActionStream = BoxStream<'static, Result<arrow_flight::Result, Status>>;
    type ListActionsStream = BoxStream<'static, Result<ActionType, Status>>;
    type DoExchangeStream = BoxStream<'static, Result<FlightData, Status>>;

    async fn handshake(
        &self,
        _request: Request<Streaming<HandshakeRequest>>,
    ) -> Result<Response<Self::HandshakeStream>, Status> {
        Err(Status::unimplemented("Implement handshake"))
    }

    async fn list_flights(
        &self,
        _request: Request<Criteria>,
    ) -> Result<Response<Self::ListFlightsStream>, Status> {
        Err(Status::unimplemented("Implement list_flights"))
    }

    async fn get_flight_info(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        Err(Status::unimplemented("Implement get_flight_info"))
    }

    async fn poll_flight_info(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<PollInfo>, Status> {
        Err(Status::unimplemented("Implement poll_flight_info"))
    }

    async fn get_schema(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<SchemaResult>, Status> {
        Err(Status::unimplemented("Implement get_schema"))
    }

    async fn do_get(
        &self,
        request: Request<Ticket>,
    ) -> Result<Response<Self::DoGetStream>, Status> {
        let (_metadata, _ext, body) = request.into_parts();
        let doget = GetCommand::decode(body.ticket).map_err(|err| {
            Status::invalid_argument(format!("Cannot decode DoGet message: {err}"))
        })?;
        let id = doget.id as usize;
        match self.data.get(&id) {
            Some(record_batches) => {
                if record_batches.len() == 0 {
                    return Ok(Response::new(Box::pin(futures::stream::empty())));
                }
                let schema = record_batches[0].schema();
                let rb_stream = futures::stream::iter(record_batches.clone()).map(Ok);
                let flight_stream = FlightDataEncoderBuilder::new()
                    .with_schema(schema)
                    .with_dictionary_handling(arrow_flight::encode::DictionaryHandling::Resend)
                    .with_max_flight_data_size(usize::MAX)
                    .build(rb_stream);

                Ok(Response::new(Box::pin(flight_stream.map_err(
                    |err| match err {
                        FlightError::Tonic(status) => *status,
                        _ => Status::internal(format!("Error during flight stream: {err}")),
                    },
                ))))
            }
            None => Ok(Response::new(Box::pin(futures::stream::empty()))),
        }
    }

    async fn do_put(
        &self,
        request: Request<Streaming<FlightData>>,
    ) -> Result<Response<Self::DoPutStream>, Status> {
        let mut request = request.map(PeekableStream::new);
        let mut stream = Pin::new(request.get_mut());

        let peeked_item = stream.peek().await.cloned();
        let Some(cmd) = peeked_item else {
            return Err(Status::internal(format!("command missing")));
        };

        let Some(flight_descriptor) = cmd?.flight_descriptor else {
            return Err(Status::internal(format!("flight descriptor missing")));
        };

        let message = PutCommand::decode(flight_descriptor.cmd)
            .map_err(|e| Status::invalid_argument(format!("unable to decode cmd {e}")))?;

        let id = message.id as usize;
        // Consume the rest of the stream
        let record_batch_stream = FlightRecordBatchStream::new_from_flight_data(
            request.into_inner().map_err(|e| e.into()),
        );
        let record_batches: Vec<RecordBatch> = record_batch_stream.try_collect().await?;
        self.data.insert(id, record_batches);

        Ok(Response::new(Box::pin(futures::stream::empty())))
    }

    async fn do_action(
        &self,
        _request: Request<Action>,
    ) -> Result<Response<Self::DoActionStream>, Status> {
        Err(Status::unimplemented("Implement do_action"))
    }

    async fn list_actions(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::ListActionsStream>, Status> {
        Err(Status::unimplemented("Implement list_actions"))
    }

    async fn do_exchange(
        &self,
        _request: Request<Streaming<FlightData>>,
    ) -> Result<Response<Self::DoExchangeStream>, Status> {
        Err(Status::unimplemented("Implement do_exchange"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:50051".parse()?;
    let service = FlightServiceImpl::new();
    let svc = FlightServiceServer::new(service);

    println!("Starting server at addr: {:?}", addr);
    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
