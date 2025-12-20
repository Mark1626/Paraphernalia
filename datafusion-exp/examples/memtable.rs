use std::sync::Arc;

use anyhow::Result;
use arrow::{
    array::{Float64Array, Int32Array, RecordBatch},
    datatypes::{DataType, Schema},
};
use datafusion::{arrow::datatypes::Field, catalog::MemTable, prelude::SessionContext};

#[tokio::main]
async fn main() -> Result<()> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("a", DataType::Int32, false),
        Field::new("b", DataType::Float64, false),
    ]));
    let batch_size = 8192;
    let batches = 10;

    let batches: Vec<RecordBatch> = (0..batches)
        .map(|i| {
            RecordBatch::try_new(
                Arc::clone(&schema),
                vec![
                    Arc::new(Int32Array::from(vec![i as i32; batch_size])),
                    Arc::new(Float64Array::from(vec![i as f64; batch_size])),
                ],
            )
        })
        .collect::<Result<_, _>>()?;

    let ctx = SessionContext::new();
    let provider = MemTable::try_new(schema, vec![batches])?;
    ctx.register_table("t", Arc::new(provider))?;

    let df = ctx.sql("select a from t limit 10").await?;
    let result = df.collect().await?;
    println!("Res {:#?}", result);

    Ok(())
}
