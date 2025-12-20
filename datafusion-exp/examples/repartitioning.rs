use std::sync::Arc;

use anyhow::{Ok, Result};
use arrow::datatypes::{DataType, Schema};
use datafusion::{
    arrow::datatypes::Field,
    catalog::memory::{DataSourceExec, MemorySourceConfig},
    common::record_batch,
    datasource::source::DataSource,
    physical_plan::{
        ExecutionPlan, ExecutionPlanProperties, Partitioning, display::DisplayableExecutionPlan,
        repartition::RepartitionExec,
    },
    prelude::SessionContext,
};
use futures_util::stream::StreamExt;

fn print_plan(plan: Arc<dyn ExecutionPlan>) {
    println!(
        "Plan {}",
        DisplayableExecutionPlan::new(plan.as_ref())
            .indent(false)
            .to_string()
    );
}

/// Notes to self
/// 1. Memory sources don't allow repartitioning to fewer partitions than originally present
/// 2. file min size is ignored for memory source
async fn memory_source_repartition() -> Result<()> {
    let schema = Arc::new(Schema::new(vec![Field::new("a", DataType::Int32, false)]));
    let partitions = vec![
        vec![
            record_batch!(("a", Int32, vec![1, 2, 3]))?,
            record_batch!(("a", Int32, vec![4, 5, 6]))?,
            record_batch!(("a", Int32, vec![1, 2, 3]))?,
            record_batch!(("a", Int32, vec![4, 5, 6]))?,
        ],
        vec![
            record_batch!(("a", Int32, vec![7, 8, 9]))?,
            record_batch!(("a", Int32, vec![10, 11, 12]))?,
        ],
        vec![record_batch!(("a", Int32, vec![13, 14, 15]))?],
        vec![
            record_batch!(("a", Int32, vec![7, 8, 9]))?,
            record_batch!(("a", Int32, vec![10, 11, 12]))?,
            record_batch!(("a", Int32, vec![7, 8, 9]))?,
            record_batch!(("a", Int32, vec![10, 11, 12]))?,
            record_batch!(("a", Int32, vec![7, 8, 9]))?,
            record_batch!(("a", Int32, vec![10, 11, 12]))?,
        ],
    ];

    let datasource: Arc<dyn DataSource> =
        Arc::new(MemorySourceConfig::try_new(&partitions, schema, None)?);

    // Default partitioning
    let plan: Arc<dyn ExecutionPlan> = Arc::new(DataSourceExec::new(Arc::clone(&datasource)));
    print_plan(plan);

    println!("--------------------------------------------");
    println!("-------Repartitioned to 4 partition --------");
    println!("--------------------------------------------");
    if let Some(datasource_repartition) = datasource.repartitioned(4, 1, None)? {
        let plan: Arc<dyn ExecutionPlan> = Arc::new(DataSourceExec::new(datasource_repartition));
        print_plan(plan);
    }

    println!("---------------------------------------------------------------");
    println!("-------Repartitioned to 5 partition, file min size 50  --------");
    println!("----------------------------------------------------------------");
    if let Some(datasource_repartition) = datasource.repartitioned(5, 1, None)? {
        let plan: Arc<dyn ExecutionPlan> = Arc::new(DataSourceExec::new(datasource_repartition));
        print_plan(plan);
    }

    println!("--------------------------------------------------------------");
    println!("-------Repartitioned to 5 partition, file min size 50 --------");
    println!("--------------------------------------------------------------");
    if let Some(datasource_repartition) = datasource.repartitioned(5, 50, None)? {
        let plan: Arc<dyn ExecutionPlan> = Arc::new(DataSourceExec::new(datasource_repartition));
        print_plan(plan);
    } else {
        println!("Unable to partition")
    }

    Ok(())
}

/// Notes to self
/// 1. Setting a high number of partitions does not mean those partitions are created
async fn repartition_exec() -> Result<()> {
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

    let datasource_exec: Arc<dyn ExecutionPlan> =
        Arc::new(DataSourceExec::new(Arc::clone(&datasource)));
    let plan: Arc<dyn ExecutionPlan> = Arc::new(RepartitionExec::try_new(
        datasource_exec,
        Partitioning::RoundRobinBatch(10),
    )?);

    let ctx = SessionContext::new();

    let n_partition = plan.output_partitioning().partition_count();
    for idx in 0..n_partition {
        println!("----------Partition {}--------------", idx + 1);
        let task_ctx = ctx.task_ctx();
        let mut stream = plan.execute(idx, task_ctx)?;
        while let Some(batch) = stream.next().await {
            println!("Batch {:?}", batch?);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    memory_source_repartition().await?;

    println!("--------------------------------------------------------------");

    repartition_exec().await?;

    Ok(())
}
