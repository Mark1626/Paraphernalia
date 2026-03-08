use std::any::Any;
use std::fmt;
use std::sync::Arc;

use arrow::datatypes::SchemaRef;
use datafusion::{
    error::DataFusionError,
    execution::{SendableRecordBatchStream, TaskContext},
    physical_expr::EquivalenceProperties,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
        execution_plan::{Boundedness, EmissionType},
        stream::RecordBatchStreamAdapter,
    },
};
use futures::StreamExt;

use crate::context::DistributedContext;
use crate::stage::Stage;

/// A leaf ExecutionPlan node that sends its encoded child plan to remote workers
/// over iroh QUIC and streams back the resulting Arrow batches.
///
/// Partitions are distributed across workers round-robin (`partition % n_workers`),
/// so there can be more output partitions than workers. In `execute(partition)`,
/// the operator retrieves the `DistributedContext` from the session config extension
/// and calls `execute_on_worker` to send the serialized plan and receive results.
#[derive(Debug)]
pub struct DistributedCoalesceExec {
    stage: Stage,
    schema: SchemaRef,
    properties: PlanProperties,
    input_plan: Arc<dyn ExecutionPlan>,
}

impl DistributedCoalesceExec {
    pub fn new(stage: Stage, input_plan: Arc<dyn ExecutionPlan>) -> Self {
        let n_partitions = input_plan.properties().partitioning.partition_count();
        let schema = input_plan.schema();
        let eq_properties = EquivalenceProperties::new(Arc::clone(&schema));
        let properties = PlanProperties::new(
            eq_properties,
            Partitioning::UnknownPartitioning(n_partitions),
            EmissionType::Final,
            Boundedness::Bounded,
        );

        Self {
            stage,
            schema,
            properties,
            input_plan,
        }
    }
}

impl DisplayAs for DistributedCoalesceExec {
    fn fmt_as(&self, t: DisplayFormatType, f: &mut fmt::Formatter) -> fmt::Result {
        match t {
            DisplayFormatType::Default | DisplayFormatType::Verbose => {
                write!(
                    f,
                    "DistributedCoalesceExec: stage_id={}",
                    self.stage.stage_id,
                )
            }
            _ => write!(f, "DistributedCoalesceExec"),
        }
    }
}

impl ExecutionPlan for DistributedCoalesceExec {
    fn name(&self) -> &str {
        "DistributedCoalesceExec"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn properties(&self) -> &PlanProperties {
        &self.properties
    }

    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![&self.input_plan]
    }

    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> datafusion::error::Result<Arc<dyn ExecutionPlan>> {
        if children.len() != 1 {
            return Err(DataFusionError::Internal(
                "DistributedCoalesceExec expects exactly one child".to_string(),
            ));
        }
        Ok(Arc::new(DistributedCoalesceExec::new(
            self.stage.clone(),
            Arc::clone(&children[0]),
        )))
    }

    fn execute(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> datafusion::error::Result<SendableRecordBatchStream> {
        let dist_ctx = context
            .session_config()
            .get_extension::<DistributedContext>()
            .ok_or_else(|| {
                DataFusionError::Internal(
                    "DistributedContext not registered in session config".to_string(),
                )
            })?;

        let worker_index = partition % dist_ctx.worker_count();

        let encoded_plan = self.stage.encoded_plan.clone();
        let schema = Arc::clone(&self.schema);

        let stream = futures::stream::once(async move {
            dist_ctx
                .execute_on_worker(worker_index, partition, &encoded_plan)
                .await
                .map_err(|e| DataFusionError::Internal(format!("{e:#}")))
        })
        .flat_map(|result| match result {
            Ok(batches) => futures::stream::iter(batches.into_iter().map(Ok)).boxed(),
            Err(e) => futures::stream::once(async move { Err(e) }).boxed(),
        });

        Ok(Box::pin(RecordBatchStreamAdapter::new(schema, stream)))
    }
}
