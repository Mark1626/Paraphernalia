use std::sync::Arc;

use anyhow::{Ok, Result};
use arrow::{
    datatypes::{DataType, Schema},
    ipc::reader::StreamReader,
};
use bytes::Bytes;
use datafusion::{
    arrow::datatypes::Field,
    catalog::{
        Session,
        memory::{DataSourceExec, MemorySourceConfig},
    },
    common::record_batch,
    datasource::source::DataSource,
    physical_plan::{
        self, ExecutionPlan, ExecutionPlanProperties, Partitioning, common::collect,
        display::DisplayableExecutionPlan, repartition::RepartitionExec,
    },
    prelude::SessionContext,
};
use futures_util::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let schema = Arc::new(Schema::new(vec![Field::new("a", DataType::Int32, false)]));
    let partitions = vec![
        vec![
            record_batch!(("a", Int32, vec![1, 2, 3]))?,
            record_batch!(("a", Int32, vec![4, 5, 6]))?,
            record_batch!(("a", Int32, vec![7, 8, 9]))?,
            record_batch!(("a", Int32, vec![10, 11, 12]))?,
        ],
        vec![
            record_batch!(("a", Int32, vec![13, 14, 15]))?,
            record_batch!(("a", Int32, vec![16, 17, 18]))?,
        ],
        vec![record_batch!(("a", Int32, vec![19, 20, 21]))?],
        vec![
            record_batch!(("a", Int32, vec![22, 23, 24]))?,
            record_batch!(("a", Int32, vec![25, 26, 27]))?,
            record_batch!(("a", Int32, vec![28, 29, 30]))?,
            record_batch!(("a", Int32, vec![31, 32, 33]))?,
            record_batch!(("a", Int32, vec![34, 35, 36]))?,
            record_batch!(("a", Int32, vec![37, 38, 39]))?,
        ],
    ];

    let datasource: Arc<dyn DataSource> =
        Arc::new(MemorySourceConfig::try_new(&partitions, schema, None)?);

    // Default partitioning
    let plan: Arc<dyn ExecutionPlan> = Arc::new(DataSourceExec::new(Arc::clone(&datasource)));

    let ctx = SessionContext::new();
    let task_ctx = ctx.task_ctx();

    let stream = physical_plan::execute_stream(plan, task_ctx)?;
    let rb = collect(stream).await?;

    // StreamReader::try_new(reader, projection)

    Ok(())
}
