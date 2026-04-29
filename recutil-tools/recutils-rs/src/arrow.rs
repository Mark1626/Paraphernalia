//! Convert rec records into Apache Arrow `RecordBatch`es.
//!
//! Gated behind the `arrow` cargo feature. Honors `%type:` declarations
//! from the rset descriptor; untyped fields fall back to `Utf8`.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanBuilder, Float64Builder, Int64Builder, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use crate::{Db, SelectionExpression};

pub fn rec_to_record_batch(
    db: &mut Db,
    record_type: &str,
) -> Result<(Arc<Schema>, RecordBatch), Box<dyn std::error::Error>> {
    let declared_types = {
        let rset = db
            .rset_by_type(record_type)
            .ok_or_else(|| format!("no record set of type {record_type:?}"))?;
        let mut types: HashMap<String, String> = HashMap::new();
        if let Some(desc) = rset.descriptor() {
            for f in desc.fields() {
                if f.name() == "%type" {
                    if let Some((field, ty)) = split_type_decl(&f.value()) {
                        types.insert(field, ty);
                    }
                }
            }
        }
        types
    };

    let (column_order, rows) = collect_rows(db, record_type)?;
    let schema = build_schema(&column_order, &declared_types);
    let columns = build_columns(&schema, &rows);
    let batch = RecordBatch::try_new(Arc::clone(&schema), columns)?;
    Ok((schema, batch))
}

/// Build a [`RecordBatch`] for the records of `record_type` that match the
/// given selection expression, using the caller-provided `schema` (so the
/// column set stays stable even when the filter excludes every record that
/// has a particular field).
pub fn rec_to_filtered_batch(
    db: &mut Db,
    record_type: &str,
    schema: &Arc<Schema>,
    selection_expression: &SelectionExpression,
) -> Result<RecordBatch, Box<dyn std::error::Error>> {
    let rset = db
        .rset_by_type(record_type)
        .ok_or_else(|| format!("no record set of type {record_type:?}"))?;

    let mut rows: Vec<HashMap<String, String>> = Vec::new();
    for (i, record) in rset.records().enumerate() {
        if !selection_expression.matches(&record) {
            continue;
        }
        let mut row: HashMap<String, String> = HashMap::new();
        for f in record.fields() {
            let name = f.name();
            if name.starts_with('%') {
                continue;
            }
            if row.contains_key(&name) {
                return Err(format!(
                    "field {:?} repeated in record {} (1-based); use a List<T> mapping (not yet supported) or remove the repeat",
                    name,
                    i + 1
                )
                .into());
            }
            row.insert(name.clone(), f.value());
        }
        rows.push(row);
    }
    let columns = build_columns(schema, &rows);
    Ok(RecordBatch::try_new(Arc::clone(schema), columns)?)
}

pub fn split_type_decl(value: &str) -> Option<(String, String)> {
    let trimmed = value.trim();
    let (name, rest) = trimmed.split_once(char::is_whitespace)?;
    Some((name.trim().to_string(), rest.trim().to_string()))
}

pub fn collect_rows(
    db: &mut Db,
    record_type: &str,
) -> Result<(Vec<String>, Vec<HashMap<String, String>>), Box<dyn std::error::Error>> {
    let rset = db
        .rset_by_type(record_type)
        .ok_or_else(|| format!("no record set of type {record_type:?}"))?;

    let mut column_order: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut rows: Vec<HashMap<String, String>> = Vec::new();

    for (i, record) in rset.records().enumerate() {
        let mut row: HashMap<String, String> = HashMap::new();
        for f in record.fields() {
            let name = f.name();
            if name.starts_with('%') {
                continue;
            }
            if row.contains_key(&name) {
                return Err(format!(
                    "field {:?} repeated in record {} (1-based); use a List<T> mapping (not yet supported) or remove the repeat",
                    name,
                    i + 1
                )
                .into());
            }
            row.insert(name.clone(), f.value());
            if seen.insert(name.clone()) {
                column_order.push(name);
            }
        }
        rows.push(row);
    }
    Ok((column_order, rows))
}

pub fn build_schema(
    column_order: &[String],
    declared: &HashMap<String, String>,
) -> Arc<Schema> {
    let fields: Vec<Field> = column_order
        .iter()
        .map(|name| {
            let dt = match declared.get(name) {
                Some(t) => map_rec_type(t),
                None => {
                    log::info!("no %type for field {name:?}; falling back to Utf8");
                    DataType::Utf8
                }
            };
            Field::new(name, dt, true)
        })
        .collect();
    Arc::new(Schema::new(fields))
}

pub fn map_rec_type(t: &str) -> DataType {
    match t.split_whitespace().next().unwrap_or("") {
        "int" | "range" => DataType::Int64,
        "real" => DataType::Float64,
        "bool" => DataType::Boolean,
        _ => DataType::Utf8,
    }
}

pub fn build_columns(schema: &Schema, rows: &[HashMap<String, String>]) -> Vec<ArrayRef> {
    schema
        .fields()
        .iter()
        .map(|f| build_column(f, rows))
        .collect()
}

pub fn build_column(field: &Field, rows: &[HashMap<String, String>]) -> ArrayRef {
    let name = field.name();
    match field.data_type() {
        DataType::Int64 => {
            let mut b = Int64Builder::with_capacity(rows.len());
            for row in rows {
                match row.get(name).map(|s| s.trim()) {
                    Some(s) if s.is_empty() => b.append_null(),
                    Some(s) => match s.parse::<i64>() {
                        Ok(v) => b.append_value(v),
                        Err(_) => {
                            log::warn!("field {name:?}: cannot parse {s:?} as int; nulled");
                            b.append_null();
                        }
                    },
                    None => b.append_null(),
                }
            }
            Arc::new(b.finish())
        }
        DataType::Float64 => {
            let mut b = Float64Builder::with_capacity(rows.len());
            for row in rows {
                match row.get(name).map(|s| s.trim()) {
                    Some(s) if s.is_empty() => b.append_null(),
                    Some(s) => match s.parse::<f64>() {
                        Ok(v) => b.append_value(v),
                        Err(_) => {
                            log::warn!("field {name:?}: cannot parse {s:?} as real; nulled");
                            b.append_null();
                        }
                    },
                    None => b.append_null(),
                }
            }
            Arc::new(b.finish())
        }
        DataType::Boolean => {
            let mut b = BooleanBuilder::with_capacity(rows.len());
            for row in rows {
                match row.get(name).map(|s| s.trim()) {
                    Some(s) if s.is_empty() => b.append_null(),
                    Some(s) => match parse_rec_bool(s) {
                        Some(v) => b.append_value(v),
                        None => {
                            log::warn!("field {name:?}: cannot parse {s:?} as bool; nulled");
                            b.append_null();
                        }
                    },
                    None => b.append_null(),
                }
            }
            Arc::new(b.finish())
        }
        DataType::Utf8 => {
            let mut b = StringBuilder::with_capacity(rows.len(), rows.len() * 16);
            for row in rows {
                match row.get(name) {
                    Some(s) => b.append_value(s),
                    None => b.append_null(),
                }
            }
            Arc::new(b.finish())
        }
        other => panic!("unsupported arrow type {other:?}"),
    }
}

pub fn parse_rec_bool(s: &str) -> Option<bool> {
    match s {
        "yes" | "true" | "1" => Some(true),
        "no" | "false" | "0" => Some(false),
        _ => None,
    }
}
