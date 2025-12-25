use anyhow::{Result, anyhow};
use arrow::array::RecordBatch;
use arrow_flight::{
    FlightClient, FlightDescriptor, PutResult, Ticket, encode::FlightDataEncoderBuilder,
};
use datafusion::common::record_batch;
use datafusion_partition_exp::flight::{GetCommand, PutCommand};
use futures::{StreamExt, TryStreamExt};
use prost::Message;
use tonic::transport::Channel;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "http://127.0.0.1:50051";
    eprintln!("Connecting to {}", addr);
    let channel = Channel::from_static(addr)
        .connect()
        .await
        .map_err(|e| anyhow!("could not create channel {e}"))?;

    let mut client = FlightClient::new(channel);

    // Submit the record batches
    let record_batches = vec![
        record_batch!(("a", Int32, vec![1, 2, 3]))?,
        record_batch!(("a", Int32, vec![4, 5, 6]))?,
    ];
    let flight_data_stream = FlightDataEncoderBuilder::new()
        .with_flight_descriptor(Some(FlightDescriptor::new_cmd(
            PutCommand { id: 0 }.encode_to_vec(),
        )))
        .build(futures::stream::iter(record_batches).map(Ok));

    let response: Vec<PutResult> = client
        .do_put(flight_data_stream)
        .await?
        .try_collect()
        .await?;
    println!("Put result {:?}", response);

    //
    let ticket = Ticket {
        ticket: GetCommand { id: 0 }.encode_to_vec().into(),
    };

    let response_stream = client.do_get(ticket).await?;
    let record_batches: Vec<RecordBatch> = response_stream.try_collect().await?;

    println!("Result {:?}", record_batches);

    Ok(())
}
