use std::collections::HashMap;
use std::path::PathBuf;

use clap::Parser;
use duckdb::arrow::record_batch::RecordBatch;
use duckdb::Connection;

use deltalake::protocol::SaveMode;
use deltalake::DeltaTable;
use url::Url;

/// Generate TPC-DS benchmark data as partitioned Delta Lake tables.
///
/// Uses DuckDB's built-in tpcds extension to generate data, then writes
/// each table as a Delta Lake table using delta-rs. Fact tables are
/// partitioned by their date key columns.
#[derive(Parser, Debug)]
#[command(name = "tpcds-delta", version)]
struct Args {
    /// TPC-DS scale factor in GB (e.g. 1, 10, 100)
    #[arg(short, long, default_value_t = 1)]
    scale_factor: u32,

    /// Output directory for Delta tables
    #[arg(short, long, default_value = "./tpcds_delta")]
    output: PathBuf,
}

/// Returns a map of table_name -> vec of partition column names.
/// Only fact tables with date keys are partitioned.
fn partition_map() -> HashMap<&'static str, Vec<&'static str>> {
    let mut m = HashMap::new();
    m.insert("store_sales", vec!["ss_sold_date_sk"]);
    m.insert("store_returns", vec!["sr_returned_date_sk"]);
    m.insert("catalog_sales", vec!["cs_sold_date_sk"]);
    m.insert("catalog_returns", vec!["cr_returned_date_sk"]);
    m.insert("web_sales", vec!["ws_sold_date_sk"]);
    m.insert("web_returns", vec!["wr_returned_date_sk"]);
    m.insert("inventory", vec!["inv_date_sk"]);
    m
}

/// Query a DuckDB table and collect all Arrow RecordBatches.
fn query_table(conn: &Connection, table_name: &str) -> Vec<RecordBatch> {
    let mut stmt = conn
        .prepare(&format!("SELECT * FROM {table_name}"))
        .expect("Failed to prepare query");

    let batches: Vec<RecordBatch> = stmt
        .query_arrow([])
        .expect("Failed to execute arrow query")
        .collect();

    batches
}

/// Write a set of RecordBatches to a Delta table, optionally partitioned.
async fn write_delta_table(
    table_path: &str,
    batches: Vec<RecordBatch>,
    partition_cols: Option<Vec<String>>,
) -> Result<DeltaTable, Box<dyn std::error::Error>> {
    let url = Url::parse(table_path)?;

    if batches.is_empty() {
        println!("    (empty table, skipping)");
        // Create an empty table so the schema is still recorded
        let table = DeltaTable::try_from_url(url)
            .await?
            .create()
            .with_save_mode(SaveMode::Overwrite)
            .await?;
        return Ok(table);
    }

    let delta_table = DeltaTable::try_from_url(url).await?;
    let mut write_builder = delta_table.write(batches).with_save_mode(SaveMode::Overwrite);

    if let Some(cols) = partition_cols {
        write_builder = write_builder.with_partition_columns(cols);
    }

    let table = write_builder.await?;
    Ok(table)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Ensure output directory exists
    std::fs::create_dir_all(&args.output)?;

    println!("=== TPC-DS Delta Lake Generator ===");
    println!("Scale factor: {} GB", args.scale_factor);
    println!("Output:       {}\n", args.output.display());

    // ── 1. Generate TPC-DS data in DuckDB ──────────────────────────────
    println!("Generating TPC-DS data in DuckDB...");
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("INSTALL tpcds; LOAD tpcds;")?;
    conn.execute_batch(&format!("CALL dsdgen(sf = {});", args.scale_factor))?;
    println!("Data generation complete.\n");

    // ── 2. Discover all tables ─────────────────────────────────────────
    let mut stmt = conn.prepare(
        "SELECT table_name FROM information_schema.tables \
         WHERE table_schema = 'main' ORDER BY table_name",
    )?;

    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    println!("Found {} tables.\n", table_names.len());

    // ── 3. Write each table as Delta ───────────────────────────────────
    let partitions = partition_map();

    for table_name in &table_names {
        let table_path = args.output.join(table_name);
        let table_uri = format!("file://{}", std::fs::canonicalize(&args.output)?.join(table_name).display());

        // Determine partition columns
        let part_cols = partitions
            .get(table_name.as_str())
            .map(|cols| cols.iter().map(|c| c.to_string()).collect::<Vec<_>>());

        let part_info = match &part_cols {
            Some(cols) => format!("partitioned by [{}]", cols.join(", ")),
            None => "unpartitioned".to_string(),
        };

        print!("  Writing {:<25} ({})...", table_name, part_info);

        // Ensure the table directory exists
        std::fs::create_dir_all(&table_path)?;

        // Query data from DuckDB as Arrow batches
        let batches = query_table(&conn, table_name);
        let row_count: usize = batches.iter().map(|b| b.num_rows()).sum();

        // Write Delta table
        match write_delta_table(&table_uri, batches, part_cols).await {
            Ok(_table) => {
                println!(" {} rows", row_count);
            }
            Err(e) => {
                println!(" ERROR: {e}");
            }
        }
    }

    println!("\nDone! Delta tables written to: {}", args.output.display());
    Ok(())
}

