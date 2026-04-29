use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use arrow::util::pretty::pretty_format_batches;
use clap::Parser;
use datafusion::prelude::SessionContext;
use recsql::RecTableProvider;

#[derive(Parser, Debug)]
#[command(
    name = "recsql",
    about = "Query a GNU recutils .rec file with SQL",
    version
)]
struct Opts {
    /// Input .rec file
    input: PathBuf,
    /// Record-set type (the name after `%rec:`); also the SQL table name
    #[arg(short = 't', long = "type")]
    record_type: String,
    /// SQL query to run
    #[arg(short = 'q', long)]
    query: String,
}

fn main() -> ExitCode {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .format_timestamp(None)
    .init();

    let opts = Opts::parse();
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("error: tokio runtime: {e}");
            return ExitCode::FAILURE;
        }
    };
    match rt.block_on(run(opts)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run(opts: Opts) -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(RecTableProvider::open(&opts.input, &opts.record_type)?);
    let ctx = SessionContext::new();
    ctx.register_table(opts.record_type.as_str(), provider)?;
    let df = ctx.sql(&opts.query).await?;
    let batches = df.collect().await?;
    println!("{}", pretty_format_batches(&batches)?);
    Ok(())
}
