use anyhow::{Context, Result};
use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::StreamWriter;
use std::io::Cursor;

/// Encode record batches into Arrow IPC stream format bytes.
///
/// Returns empty vec if there are no batches.
pub fn encode_batches(batches: &[RecordBatch]) -> Result<Vec<u8>> {
    let schema = match batches.first() {
        Some(batch) => batch.schema(),
        None => return Ok(Vec::new()),
    };
    encode_batches_with_schema(&schema, batches)
}

/// Encode record batches with an explicit schema.
pub fn encode_batches_with_schema(
    schema: &SchemaRef,
    batches: &[RecordBatch],
) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut writer =
        StreamWriter::try_new(&mut buf, schema).context("failed to create IPC writer")?;
    for batch in batches {
        writer.write(batch).context("failed to write batch")?;
    }
    writer.finish().context("failed to finish IPC stream")?;
    Ok(buf)
}

/// Decode record batches from Arrow IPC stream format bytes.
///
/// Returns empty vec for empty input.
pub fn decode_batches(bytes: &[u8]) -> Result<Vec<RecordBatch>> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let reader =
        StreamReader::try_new(Cursor::new(bytes), None).context("failed to create IPC reader")?;
    reader
        .collect::<Result<Vec<_>, _>>()
        .context("failed to read IPC batches")
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Array, Float64Array, Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    fn sample_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, true),
            Field::new("value", DataType::Float64, false),
        ]));
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3])),
                Arc::new(StringArray::from(vec![Some("alice"), None, Some("carol")])),
                Arc::new(Float64Array::from(vec![10.5, 20.0, 30.75])),
            ],
        )
        .unwrap()
    }

    #[test]
    fn roundtrip_single_batch() {
        let batch = sample_batch();
        let encoded = encode_batches(&[batch.clone()]).unwrap();
        let decoded = decode_batches(&encoded).unwrap();

        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0], batch);
    }

    #[test]
    fn roundtrip_multiple_batches() {
        let b1 = sample_batch();
        let b2 = sample_batch();
        let encoded = encode_batches(&[b1.clone(), b2.clone()]).unwrap();
        let decoded = decode_batches(&encoded).unwrap();

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0], b1);
        assert_eq!(decoded[1], b2);
    }

    #[test]
    fn roundtrip_empty_batches() {
        let encoded = encode_batches(&[]).unwrap();
        assert!(encoded.is_empty());
        let decoded = decode_batches(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn roundtrip_with_explicit_schema() {
        let batch = sample_batch();
        let schema = batch.schema();
        let encoded = encode_batches_with_schema(&schema, &[batch.clone()]).unwrap();
        let decoded = decode_batches(&encoded).unwrap();

        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].schema(), schema);
        assert_eq!(decoded[0], batch);
    }

    #[test]
    fn encode_with_schema_no_batches() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int32, false)]));
        let encoded = encode_batches_with_schema(&schema, &[]).unwrap();
        // Should produce a valid IPC stream with just schema + footer, no batches
        let decoded = decode_batches(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn decode_invalid_bytes() {
        let result = decode_batches(b"this is not valid IPC");
        assert!(result.is_err());
    }

    #[test]
    fn preserves_nulls() {
        let batch = sample_batch();
        let encoded = encode_batches(&[batch]).unwrap();
        let decoded = decode_batches(&encoded).unwrap();

        let names = decoded[0]
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert!(names.is_valid(0)); // "alice"
        assert!(names.is_null(1)); // null
        assert!(names.is_valid(2)); // "carol"
    }
}
