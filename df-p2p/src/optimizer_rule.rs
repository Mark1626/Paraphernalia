use std::sync::Arc;

use datafusion::{
    common::tree_node::{Transformed, TreeNode, TreeNodeRecursion},
    error::Result,
    physical_plan::{ExecutionPlan, coalesce_partitions::CoalescePartitionsExec},
};
use datafusion_proto::bytes::physical_plan_to_bytes;

use crate::operators::distributed_coalesce::DistributedCoalesceExec;
use crate::stage::Stage;

/// Walks the physical plan tree top-down and inserts a `DistributedCoalesceExec`
/// between the first `CoalescePartitionsExec` and its child. The child subtree
/// is serialized into a `Stage` for remote execution.
///
/// Before: `CoalescePartitionsExec → child_plan`
/// After:  `CoalescePartitionsExec → DistributedCoalesceExec → child_plan`
///
/// The output partition count of `DistributedCoalesceExec` matches the child
/// plan's partition count; partitions are distributed across available workers
/// round-robin at execution time.
pub fn apply_network_boundaries(plan: Arc<dyn ExecutionPlan>) -> Result<Arc<dyn ExecutionPlan>> {
    let transformed = plan.transform_down(|node| {
        if node
            .as_any()
            .downcast_ref::<CoalescePartitionsExec>()
            .is_some()
        {
            let child = Arc::clone(node.children()[0]);
            let plan_bytes = physical_plan_to_bytes(Arc::clone(&child))?;

            let stage = Stage {
                stage_id: 0,
                encoded_plan: plan_bytes,
            };

            let dist_exec: Arc<dyn ExecutionPlan> =
                Arc::new(DistributedCoalesceExec::new(stage, child));

            // Re-parent CoalescePartitionsExec with DistributedCoalesceExec as its child
            let new_coalesce = node.with_new_children(vec![dist_exec])?;

            Ok(Transformed::new(
                new_coalesce,
                true,
                TreeNodeRecursion::Stop,
            ))
        } else {
            Ok(Transformed::no(node))
        }
    })?;

    Ok(transformed.data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Int32Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use datafusion::datasource::memory::MemorySourceConfig;
    use datafusion::physical_plan::coalesce_partitions::CoalescePartitionsExec;
    use datafusion::physical_plan::display::DisplayableExecutionPlan;
    use datafusion_datasource::source::DataSourceExec;

    fn display_plan(plan: &dyn ExecutionPlan) -> String {
        DisplayableExecutionPlan::new(plan)
            .indent(false)
            .to_string()
    }

    /// Build a DataSourceExec backed by in-memory data with the given number of partitions.
    fn mem_exec(n_partitions: usize) -> Arc<dyn ExecutionPlan> {
        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int32, false)]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int32Array::from(vec![1, 2, 3]))],
        )
        .unwrap();
        let partitions: Vec<Vec<RecordBatch>> =
            (0..n_partitions).map(|_| vec![batch.clone()]).collect();
        DataSourceExec::from_data_source(
            MemorySourceConfig::try_new(&partitions, schema, None).unwrap(),
        )
    }

    #[test]
    fn snapshot_coalesce_replaced() {
        let child = mem_exec(4);
        let coalesce: Arc<dyn ExecutionPlan> = Arc::new(CoalescePartitionsExec::new(child));

        let distributed = apply_network_boundaries(coalesce).unwrap();
        insta::assert_snapshot!(display_plan(distributed.as_ref()), @r"
        CoalescePartitionsExec
          DistributedCoalesceExec: stage_id=0
            DataSourceExec: partitions=4, partition_sizes=[1, 1, 1, 1]
        ");
    }

    #[test]
    fn snapshot_no_coalesce_unchanged() {
        let plan = mem_exec(2);

        let distributed = apply_network_boundaries(plan).unwrap();
        insta::assert_snapshot!(display_plan(distributed.as_ref()), @"DataSourceExec: partitions=2, partition_sizes=[1, 1]");
    }

    #[test]
    fn snapshot_only_first_coalesce_replaced() {
        // outer CoalescePartitionsExec → inner CoalescePartitionsExec → DataSourceExec
        let leaf = mem_exec(4);
        let inner_coalesce: Arc<dyn ExecutionPlan> = Arc::new(CoalescePartitionsExec::new(leaf));
        let outer_coalesce: Arc<dyn ExecutionPlan> =
            Arc::new(CoalescePartitionsExec::new(inner_coalesce));

        let distributed = apply_network_boundaries(outer_coalesce).unwrap();
        insta::assert_snapshot!(display_plan(distributed.as_ref()), @r"
        CoalescePartitionsExec
          DistributedCoalesceExec: stage_id=0
            CoalescePartitionsExec
              DataSourceExec: partitions=4, partition_sizes=[1, 1, 1, 1]
        ");
    }
}
