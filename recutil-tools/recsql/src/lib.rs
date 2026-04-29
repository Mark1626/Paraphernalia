//! Query GNU recutils `.rec` files with SQL via Apache DataFusion.
//!
//! [`RecTableProvider`] exposes one record set as a `TableProvider`.
//! The rec file is parsed eagerly when the provider is opened; SQL queries
//! run against the cached `RecordBatch`. If the file changes on disk after
//! `open()`, the change is not reflected until the provider is rebuilt.

use std::any::Any;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use datafusion::catalog::{Session, TableProvider};
use datafusion::datasource::MemTable;
use datafusion::error::Result as DfResult;
use datafusion::logical_expr::{Expr, TableType};
use datafusion::physical_plan::ExecutionPlan;
use recutils_rs::Db;
use recutils_rs::arrow::rec_to_record_batch;

#[derive(Debug)]
pub struct RecTableProvider {
    inner: MemTable,
}

impl RecTableProvider {
    pub fn open<P: AsRef<Path>>(
        path: P,
        record_type: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let text = fs::read_to_string(path.as_ref())?;
        let mut db = Db::parse_str(&text)?;
        let (schema, batch) = rec_to_record_batch(&mut db, record_type)?;
        Self::from_batch(schema, batch).map_err(Into::into)
    }

    pub fn from_batch(
        schema: SchemaRef,
        batch: RecordBatch,
    ) -> DfResult<Self> {
        let inner = MemTable::try_new(schema, vec![vec![batch]])?;
        Ok(Self { inner })
    }
}

#[async_trait]
impl TableProvider for RecTableProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.inner.schema()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> DfResult<Arc<dyn ExecutionPlan>> {
        self.inner.scan(state, projection, filters, limit).await
    }
}
