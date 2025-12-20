use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use anyhow::Result;
use arrow::datatypes::{DataType, Field, Schema};
use datafusion::common::record_batch;
use datafusion::physical_plan::memory::MemoryStream;
use futures::{Stream, stream};
use futures_util::stream::StreamExt;
use pin_project::pin_project;

#[pin_project]
pub struct RepeatingStream<S, I> {
    #[pin]
    inner: S,
    times: usize,
    count: usize,
    current: Option<I>,
}

impl<S, I> RepeatingStream<S, I> {
    pub fn new(inner: S, times: usize) -> Self {
        assert!(times > 0);
        Self {
            inner,
            times,
            count: 0,
            current: None,
        }
    }
}

impl<S, I> Stream for RepeatingStream<S, I>
where
    S: Stream<Item = I> + Unpin,
    I: Clone,
{
    type Item = I;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if *this.count > 0 {
            *this.count -= 1;
            return Poll::Ready(this.current.clone());
        } else {
            match this.inner.poll_next(cx) {
                Poll::Pending => {
                    return Poll::Pending;
                }
                Poll::Ready(Some(e)) => {
                    *this.count = *this.times - 1;
                    *this.current = Some(e.clone());
                    return Poll::Ready(Some(e));
                }
                Poll::Ready(None) => {
                    return Poll::Ready(None);
                }
            }
        }
    }
}

async fn custom_stream_test() {
    eprintln!("Stream");
    let mut istream = stream::iter(1..=3);
    let istream_cpy = istream.clone();
    while let Some(v) = istream.next().await {
        eprint!("{:?} ", v);
    }

    eprintln!("\nRepeat Stream");
    let mut stream = RepeatingStream::new(istream_cpy, 3);
    while let Some(v) = stream.next().await {
        eprint!("{:?} ", v);
    }
}

async fn mem_rb_streams() -> Result<()> {
    let record_batches = vec![
        record_batch!(("a", Int32, vec![1, 2, 3]))?,
        record_batch!(("a", Int32, vec![4, 5, 6]))?,
        record_batch!(("a", Int32, vec![7, 8, 9]))?,
        record_batch!(("a", Int32, vec![10, 11, 12]))?,
    ];
    let schema = Arc::new(Schema::new(vec![Field::new("a", DataType::Int32, false)]));
    let mut stream = Box::pin(MemoryStream::try_new(record_batches, schema, None)?);

    let mut batches = vec![];
    while let Some(record_batch) = stream.next().await {
        batches.push(record_batch?);
    }

    eprintln!("batches = {:?}", batches);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    mem_rb_streams().await?;

    eprintln!("\n------------");

    custom_stream_test().await;

    eprintln!("\n------------");
    Ok(())
}
