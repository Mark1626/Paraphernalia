//! Generate an Iceberg table with hidden partition transforms.
//!
//! This creates a local Iceberg table using the MemoryCatalog backed by the
//! local filesystem via `LocalFsStorageFactory`. The table has hidden
//! partitions using transforms:
//! - `identity(event_id)` — identity transform on event_id
//! - `day(event_ts)` — extracts the day from a timestamp column
//! - `bucket[4](user_id)` — hashes user_id into 4 buckets
//! - `truncate[3](city)` — truncates city to 3 characters
//!
//! Data is written as Parquet files via the iceberg-rust writer API, then
//! committed via a Transaction + fast_append.

use std::collections::HashMap;
use std::sync::Arc;

use arrow_array::{Int32Array, Int64Array, RecordBatch, StringArray, TimestampMicrosecondArray};
use arrow_schema::{DataType, Field, TimeUnit};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use iceberg::io::LocalFsStorageFactory;
use iceberg::memory::{MEMORY_CATALOG_WAREHOUSE, MemoryCatalogBuilder};
use iceberg::spec::{
    DataFileFormat, Literal, NestedField, PartitionKey, PrimitiveType, Schema, Struct, Transform,
    Type, UnboundPartitionSpec,
};
use iceberg::transaction::{ApplyTransactionAction, Transaction};
use iceberg::writer::base_writer::data_file_writer::DataFileWriterBuilder;
use iceberg::writer::file_writer::ParquetWriterBuilder;
use iceberg::writer::file_writer::location_generator::{
    DefaultFileNameGenerator, DefaultLocationGenerator,
};
use iceberg::writer::file_writer::rolling_writer::RollingFileWriterBuilder;
use iceberg::writer::partitioning::PartitioningWriter;
use iceberg::writer::partitioning::fanout_writer::FanoutWriter;
use iceberg::{Catalog, CatalogBuilder, NamespaceIdent, TableCreation};
use parquet::arrow::PARQUET_FIELD_ID_META_KEY;
use parquet::file::properties::WriterProperties;

/// Output directory for the Iceberg warehouse (relative to cwd).
const WAREHOUSE_PATH: &str = "warehouse";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ---------------------------------------------------------------
    // 1. Resolve an absolute warehouse path on the local filesystem
    // ---------------------------------------------------------------
    let cwd = std::env::current_dir()?;
    let warehouse_dir = cwd.join(WAREHOUSE_PATH);
    // Clean up any previous run
    if warehouse_dir.exists() {
        std::fs::remove_dir_all(&warehouse_dir)?;
    }
    std::fs::create_dir_all(&warehouse_dir)?;
    let warehouse_url = format!("file://{}", warehouse_dir.display());
    println!("Warehouse location: {warehouse_url}");

    // ---------------------------------------------------------------
    // 2. Create an in-memory catalog backed by the local filesystem
    // ---------------------------------------------------------------
    // IMPORTANT: Inject LocalFsStorageFactory so that Parquet files,
    // metadata.json, manifests, etc. are written to the real filesystem
    // instead of staying in memory.
    let catalog = MemoryCatalogBuilder::default()
        .with_storage_factory(Arc::new(LocalFsStorageFactory))
        .load(
            "local",
            HashMap::from([(MEMORY_CATALOG_WAREHOUSE.to_string(), warehouse_url)]),
        )
        .await?;

    // Create a namespace
    let namespace = NamespaceIdent::from_strs(["test_db"])?;
    catalog
        .create_namespace(&namespace, HashMap::new())
        .await?;

    // ---------------------------------------------------------------
    // 3. Define the table schema
    // ---------------------------------------------------------------
    // Field IDs must be unique and start from 1.
    let schema = Schema::builder()
        .with_schema_id(0)
        .with_fields(vec![
            NestedField::required(1, "event_id", Type::Primitive(PrimitiveType::Int)).into(),
            NestedField::required(
                2,
                "event_ts",
                Type::Primitive(PrimitiveType::Timestamptz),
            )
            .into(),
            NestedField::required(3, "user_id", Type::Primitive(PrimitiveType::Long)).into(),
            NestedField::required(4, "city", Type::Primitive(PrimitiveType::String)).into(),
            NestedField::optional(5, "payload", Type::Primitive(PrimitiveType::String)).into(),
        ])
        .build()?;

    // ---------------------------------------------------------------
    // 4. Define hidden partition spec with multiple transforms
    // ---------------------------------------------------------------
    // These are "hidden" partitions: users query the source columns
    // (event_id, event_ts, user_id, city) and the engine automatically
    // derives partition predicates via the transforms.
    let partition_spec = UnboundPartitionSpec::builder()
        .with_spec_id(0)
        .add_partition_field(1, "event_id_identity".to_string(), Transform::Identity)?
        .add_partition_field(2, "event_ts_day".to_string(), Transform::Day)?
        .add_partition_field(3, "user_id_bucket".to_string(), Transform::Bucket(4))?
        .add_partition_field(4, "city_truncate".to_string(), Transform::Truncate(3))?
        .build();

    println!("Partition spec: {partition_spec:?}");

    // ---------------------------------------------------------------
    // 5. Create the Iceberg table via the catalog
    // ---------------------------------------------------------------
    let table_creation = TableCreation::builder()
        .name("events".to_string())
        .schema(schema)
        .partition_spec(partition_spec)
        .build();

    let table = catalog.create_table(&namespace, table_creation).await?;

    println!(
        "Created table: {} at {}",
        table.identifier(),
        table.metadata().location()
    );
    println!(
        "Default partition spec: {:?}",
        table.metadata().default_partition_spec()
    );

    // ---------------------------------------------------------------
    // 6. Set up the writer pipeline with FanoutWriter for partitioning
    // ---------------------------------------------------------------
    let location_generator = DefaultLocationGenerator::new(table.metadata().clone())?;
    let file_name_generator = DefaultFileNameGenerator::new(
        "data".to_string(),
        None,
        DataFileFormat::Parquet,
    );

    let parquet_writer_builder = ParquetWriterBuilder::new(
        WriterProperties::builder().build(),
        table.metadata().current_schema().clone(),
    );

    let rolling_writer_builder = RollingFileWriterBuilder::new_with_default_file_size(
        parquet_writer_builder,
        table.file_io().clone(),
        location_generator,
        file_name_generator,
    );

    let data_file_writer_builder = DataFileWriterBuilder::new(rolling_writer_builder);

    // Use FanoutWriter to handle multiple partitions simultaneously
    let mut fanout_writer = FanoutWriter::new(data_file_writer_builder);

    // ---------------------------------------------------------------
    // 7. Build Arrow RecordBatches with sample data and write per partition
    // ---------------------------------------------------------------
    let arrow_schema = Arc::new(arrow_schema::Schema::new(vec![
        Field::new("event_id", DataType::Int32, false).with_metadata(HashMap::from([(
            PARQUET_FIELD_ID_META_KEY.to_string(),
            "1".to_string(),
        )])),
        Field::new(
            "event_ts",
            DataType::Timestamp(TimeUnit::Microsecond, Some("+00:00".into())),
            false,
        )
        .with_metadata(HashMap::from([(
            PARQUET_FIELD_ID_META_KEY.to_string(),
            "2".to_string(),
        )])),
        Field::new("user_id", DataType::Int64, false).with_metadata(HashMap::from([(
            PARQUET_FIELD_ID_META_KEY.to_string(),
            "3".to_string(),
        )])),
        Field::new("city", DataType::Utf8, false).with_metadata(HashMap::from([(
            PARQUET_FIELD_ID_META_KEY.to_string(),
            "4".to_string(),
        )])),
        Field::new("payload", DataType::Utf8, true).with_metadata(HashMap::from([(
            PARQUET_FIELD_ID_META_KEY.to_string(),
            "5".to_string(),
        )])),
    ]));

    // Sample data rows -- we write them one at a time to assign correct
    // partition keys per row. In production you would batch rows that
    // share the same partition key.
    let rows: Vec<RowData> = vec![
        RowData {
            event_id: 1,
            event_ts: make_ts(2024, 1, 15, 10, 0, 0),
            user_id: 100,
            city: "San Francisco",
            payload: Some("login"),
        },
        RowData {
            event_id: 2,
            event_ts: make_ts(2024, 1, 15, 14, 30, 0),
            user_id: 200,
            city: "New York",
            payload: Some("purchase"),
        },
        RowData {
            event_id: 3,
            event_ts: make_ts(2024, 1, 16, 9, 0, 0),
            user_id: 100,
            city: "San Francisco",
            payload: Some("logout"),
        },
        RowData {
            event_id: 4,
            event_ts: make_ts(2024, 1, 16, 18, 0, 0),
            user_id: 300,
            city: "Chicago",
            payload: None,
        },
        RowData {
            event_id: 5,
            event_ts: make_ts(2024, 2, 1, 12, 0, 0),
            user_id: 200,
            city: "New York",
            payload: Some("purchase"),
        },
        RowData {
            event_id: 6,
            event_ts: make_ts(2024, 2, 1, 15, 0, 0),
            user_id: 400,
            city: "Seattle",
            payload: Some("signup"),
        },
        RowData {
            event_id: 7,
            event_ts: make_ts(2024, 3, 10, 8, 0, 0),
            user_id: 100,
            city: "San Francisco",
            payload: Some("login"),
        },
        RowData {
            event_id: 8,
            event_ts: make_ts(2024, 3, 10, 20, 0, 0),
            user_id: 300,
            city: "Chicago",
            payload: Some("purchase"),
        },
    ];

    let bound_partition_spec = table.metadata().default_partition_spec().as_ref().clone();
    let iceberg_schema = table.metadata().current_schema().clone();

    println!("Writing {} rows...", rows.len());

    for row in &rows {
        // Build a single-row RecordBatch
        let batch = RecordBatch::try_new(arrow_schema.clone(), vec![
            Arc::new(Int32Array::from(vec![row.event_id])),
            Arc::new(
                TimestampMicrosecondArray::from(vec![row.event_ts]).with_timezone("+00:00"),
            ),
            Arc::new(Int64Array::from(vec![row.user_id])),
            Arc::new(StringArray::from(vec![row.city])),
            Arc::new(StringArray::from(vec![row.payload])),
        ])?;

        // Compute partition values for this row using the transforms:
        //   event_id_identity: Identity(event_id) -> event_id as int
        //   event_ts_day:      Day(event_ts)      -> days since epoch
        //   user_id_bucket:    Bucket(4)(user_id)  -> hash(user_id) % 4
        //   city_truncate:     Truncate(3)(city)   -> first 3 chars
        let partition_values = Struct::from_iter(vec![
            Some(Literal::int(row.event_id)),
            Some(Literal::int(days_since_epoch(row.event_ts))),
            Some(Literal::int(bucket_hash_long(row.user_id, 4))),
            Some(Literal::string(truncate_str(row.city, 3))),
        ]);

        let partition_key = PartitionKey::new(
            bound_partition_spec.clone(),
            iceberg_schema.clone(),
            partition_values,
        );

        fanout_writer.write(partition_key, batch).await?;
    }

    // ---------------------------------------------------------------
    // 8. Close writer and get data files
    // ---------------------------------------------------------------
    let data_files = fanout_writer.close().await?;
    println!("Wrote {} data file(s):", data_files.len());
    for df in &data_files {
        println!(
            "  - {} ({} bytes, format: {:?})",
            df.file_path(),
            df.file_size_in_bytes(),
            df.file_format()
        );
    }

    // ---------------------------------------------------------------
    // 9. Commit the data files via a Transaction
    // ---------------------------------------------------------------
    let tx = Transaction::new(&table);
    let action = tx.fast_append().add_data_files(data_files);
    let tx = action.apply(tx)?;
    let table = tx.commit(&catalog).await?;

    println!(
        "Committed! Table now has {} snapshot(s)",
        table.metadata().snapshots().count()
    );

    // Print the metadata file location
    if let Some(loc) = table.metadata_location() {
        println!("Metadata location: {loc}");
    }

    println!("\nDone! The Iceberg table with hidden partitions is at:");
    println!("  {}", table.metadata().location());

    Ok(())
}

// =================================================================
// Helper types and functions
// =================================================================

struct RowData {
    event_id: i32,
    event_ts: i64,
    user_id: i64,
    city: &'static str,
    payload: Option<&'static str>,
}

/// Convert a date/time into microseconds since epoch (UTC).
fn make_ts(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> i64 {
    let dt = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date"),
        NaiveTime::from_hms_opt(hour, min, sec).expect("valid time"),
    );
    dt.and_utc().timestamp_micros()
}

/// Compute day partition value: days since Unix epoch from a microsecond timestamp.
/// This matches the Iceberg `Day` transform for timestamptz.
fn days_since_epoch(ts_micros: i64) -> i32 {
    // Iceberg Day transform: microseconds -> days since epoch
    // Integer division truncating toward zero for positive values
    let secs = ts_micros / 1_000_000;
    let days = secs / 86400;
    days as i32
}

/// Compute bucket partition value: murmur3 hash of a long value mod N.
/// This matches the Iceberg `Bucket(N)` transform.
fn bucket_hash_long(value: i64, num_buckets: u32) -> i32 {
    let hash = murmur3_x86_32(&value.to_le_bytes(), 0);
    ((hash & i32::MAX as u32) % num_buckets) as i32
}

/// Truncate a string to width characters.
/// This matches the Iceberg `Truncate(W)` transform for strings.
fn truncate_str(s: &str, width: usize) -> String {
    s.chars().take(width).collect()
}

/// Murmur3 x86 32-bit hash (the variant used by Iceberg).
fn murmur3_x86_32(data: &[u8], seed: u32) -> u32 {
    let c1: u32 = 0xcc9e_2d51;
    let c2: u32 = 0x1b87_3593;

    let mut h1 = seed;
    let len = data.len();
    let n_blocks = len / 4;

    // Body
    for i in 0..n_blocks {
        let offset = i * 4;
        let k = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);

        let mut k1 = k;
        k1 = k1.wrapping_mul(c1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(c2);

        h1 ^= k1;
        h1 = h1.rotate_left(13);
        h1 = h1.wrapping_mul(5).wrapping_add(0xe654_6b64);
    }

    // Tail
    let tail = &data[n_blocks * 4..];
    let mut k1: u32 = 0;
    match tail.len() {
        3 => {
            k1 ^= (tail[2] as u32) << 16;
            k1 ^= (tail[1] as u32) << 8;
            k1 ^= tail[0] as u32;
            k1 = k1.wrapping_mul(c1);
            k1 = k1.rotate_left(15);
            k1 = k1.wrapping_mul(c2);
            h1 ^= k1;
        }
        2 => {
            k1 ^= (tail[1] as u32) << 8;
            k1 ^= tail[0] as u32;
            k1 = k1.wrapping_mul(c1);
            k1 = k1.rotate_left(15);
            k1 = k1.wrapping_mul(c2);
            h1 ^= k1;
        }
        1 => {
            k1 ^= tail[0] as u32;
            k1 = k1.wrapping_mul(c1);
            k1 = k1.rotate_left(15);
            k1 = k1.wrapping_mul(c2);
            h1 ^= k1;
        }
        _ => {}
    }

    // Finalization
    h1 ^= len as u32;
    h1 ^= h1 >> 16;
    h1 = h1.wrapping_mul(0x85eb_ca6b);
    h1 ^= h1 >> 13;
    h1 = h1.wrapping_mul(0xc2b2_ae35);
    h1 ^= h1 >> 16;

    h1
}
