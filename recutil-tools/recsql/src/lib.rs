//! Query GNU recutils `.rec` files with SQL via Apache DataFusion.
//!
//! [`RecTableProvider`] exposes one record set as a `TableProvider`. The rec
//! file's source is held inside the provider so each scan can re-parse it
//! and apply a selection-expression filter at the librec layer when
//! DataFusion pushes predicates down.
//!
//! Filter pushdown is best-effort: predicates that translate to a selection
//! expression are reported as `Inexact` (DataFusion still re-checks above
//! us); everything else is `Unsupported` and handled in DataFusion as
//! before.

mod pushdown;

use std::any::Any;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use datafusion::catalog::{Session, TableProvider};
use datafusion::datasource::MemTable;
use datafusion::error::{DataFusionError, Result as DfResult};
use datafusion::logical_expr::{Expr, TableProviderFilterPushDown, TableType};
use datafusion::physical_plan::ExecutionPlan;
use recutils_rs::Db;
use recutils_rs::SelectionExpression;
use recutils_rs::arrow::{rec_to_filtered_batch, rec_to_record_batch};

use crate::pushdown::expr_to_selection_expression;

#[derive(Debug)]
pub struct RecTableProvider {
    source: String,
    record_type: String,
    schema: SchemaRef,
    cached: RecordBatch,
}

impl RecTableProvider {
    pub fn open<P: AsRef<Path>>(
        path: P,
        record_type: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let source = fs::read_to_string(path.as_ref())?;
        let mut db = Db::parse_str(&source)?;
        let (schema, cached) = rec_to_record_batch(&mut db, record_type)?;
        Ok(Self {
            source,
            record_type: record_type.to_string(),
            schema,
            cached,
        })
    }

    fn batches_for(&self, filters: &[Expr]) -> DfResult<Vec<RecordBatch>> {
        let clauses: Vec<String> = filters
            .iter()
            .filter_map(|f| expr_to_selection_expression(f, self.schema.as_ref()))
            .collect();

        if clauses.is_empty() {
            return Ok(vec![self.cached.clone()]);
        }

        let combined = if clauses.len() == 1 {
            clauses.into_iter().next().unwrap()
        } else {
            clauses
                .into_iter()
                .map(|c| format!("({c})"))
                .collect::<Vec<_>>()
                .join(" && ")
        };

        let selection_expression = match SelectionExpression::compile(&combined, false) {
            Ok(s) => {
                log::debug!("pushed selection expression to librec: {combined}");
                s
            }
            Err(e) => {
                log::warn!(
                    "selection expression compile failed for pushdown expression {combined:?}: \
                     {e}; falling back to unfiltered scan"
                );
                return Ok(vec![self.cached.clone()]);
            }
        };

        let mut db = Db::parse_str(&self.source)
            .map_err(|e| DataFusionError::Execution(e.to_string()))?;
        let batch = rec_to_filtered_batch(
            &mut db,
            &self.record_type,
            &self.schema,
            &selection_expression,
        )
        .map_err(|e| DataFusionError::Execution(e.to_string()))?;
        Ok(vec![batch])
    }
}

#[async_trait]
impl TableProvider for RecTableProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> DfResult<Vec<TableProviderFilterPushDown>> {
        Ok(filters
            .iter()
            .map(|f| {
                if expr_to_selection_expression(f, self.schema.as_ref()).is_some() {
                    TableProviderFilterPushDown::Inexact
                } else {
                    TableProviderFilterPushDown::Unsupported
                }
            })
            .collect())
    }

    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> DfResult<Arc<dyn ExecutionPlan>> {
        let batches = self.batches_for(filters)?;
        let mem = MemTable::try_new(Arc::clone(&self.schema), vec![batches])?;
        mem.scan(state, projection, filters, limit).await
    }
}
