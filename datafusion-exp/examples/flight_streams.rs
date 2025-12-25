use anyhow::Result;
use arrow::array::RecordBatch;
use std::sync::Arc;

use arrow::datatypes::{DataType, Field, Schema};
use arrow_flight::{decode::FlightRecordBatchStream, encode::FlightDataEncoderBuilder};
use datafusion::common::record_batch;
use futures::StreamExt;
use futures_util::stream::TryStreamExt;

async fn flight_stream_creation(
    record_batches: Vec<RecordBatch>,
    schema: Arc<Schema>,
) -> Result<FlightRecordBatchStream> {
    let stream = futures::stream::iter(record_batches).map(Ok);
    let flight_data = FlightDataEncoderBuilder::new()
        .with_schema(schema)
        .with_dictionary_handling(arrow_flight::encode::DictionaryHandling::Resend)
        .with_max_flight_data_size(usize::MAX)
        .build(stream);

    let flight_stream = FlightRecordBatchStream::new_from_flight_data(flight_data);

    Ok(flight_stream)
}

#[tokio::main]
async fn main() -> Result<()> {
    let schema = Arc::new(Schema::new(vec![Field::new("a", DataType::Int32, false)]));
    let record_batches = vec![
        record_batch!(("a", Int32, vec![1, 2, 3]))?,
        record_batch!(("a", Int32, vec![4, 5, 6]))?,
    ];

    let flight_stream = flight_stream_creation(record_batches.clone(), schema).await?;
    let actual_record_batches: Vec<RecordBatch> = flight_stream.try_collect().await?;

    println!("Record batches {:#?}", actual_record_batches);

    Ok(())
}
