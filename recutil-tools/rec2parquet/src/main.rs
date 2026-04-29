use std::fs::{self, File};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use recutils_rs::Db;
use recutils_rs::arrow as rec_arrow;

#[derive(Parser, Debug)]
#[command(
    name = "rec2parquet",
    about = "Convert a GNU recutils .rec file to Apache Parquet",
    version
)]
struct Opts {
    /// Input .rec file
    input: PathBuf,
    /// Output .parquet file
    output: PathBuf,
    /// Record-set type to convert (the name after `%rec:`)
    #[arg(short = 't', long = "type")]
    record_type: String,
    /// Compression codec
    #[arg(short = 'c', long, value_enum, default_value_t = CompressionArg::Uncompressed)]
    compression: CompressionArg,
    /// Maximum number of rows per row group
    #[arg(long, default_value_t = 1_048_576)]
    max_row_group_size: usize,
    /// Print the inferred schema before writing
    #[arg(short = 'p', long = "print-schema")]
    print_schema: bool,
    /// Print the inferred schema and exit (no parquet output written)
    #[arg(short = 'n', long)]
    dry: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum CompressionArg {
    Uncompressed,
    Snappy,
    Gzip,
    Lzo,
    Brotli,
    Lz4,
    Zstd,
    Lz4Raw,
}

impl CompressionArg {
    fn to_parquet(self) -> Compression {
        use parquet::basic::{BrotliLevel, GzipLevel, ZstdLevel};
        match self {
            CompressionArg::Uncompressed => Compression::UNCOMPRESSED,
            CompressionArg::Snappy => Compression::SNAPPY,
            CompressionArg::Gzip => Compression::GZIP(GzipLevel::default()),
            CompressionArg::Lzo => Compression::LZO,
            CompressionArg::Brotli => Compression::BROTLI(BrotliLevel::default()),
            CompressionArg::Lz4 => Compression::LZ4,
            CompressionArg::Zstd => Compression::ZSTD(ZstdLevel::default()),
            CompressionArg::Lz4Raw => Compression::LZ4_RAW,
        }
    }
}

fn main() -> ExitCode {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .format_timestamp(None)
    .init();

    let opts = Opts::parse();
    match run(opts) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(opts: Opts) -> Result<(), Box<dyn std::error::Error>> {
    let text = fs::read_to_string(&opts.input)
        .map_err(|e| format!("reading {}: {e}", opts.input.display()))?;
    let mut db = Db::parse_str(&text)?;

    let (schema, batch) = rec_arrow::rec_to_record_batch(&mut db, &opts.record_type)?;

    if opts.print_schema || opts.dry {
        println!("{schema:#?}");
        if opts.dry {
            return Ok(());
        }
    }

    let props = WriterProperties::builder()
        .set_compression(opts.compression.to_parquet())
        .set_max_row_group_row_count(Some(opts.max_row_group_size))
        .build();
    let file = File::create(&opts.output)
        .map_err(|e| format!("creating {}: {e}", opts.output.display()))?;
    let mut writer = ArrowWriter::try_new(file, schema, Some(props))?;
    writer.write(&batch)?;
    writer.close()?;

    println!(
        "wrote {} record(s), {} column(s) to {}",
        batch.num_rows(),
        batch.num_columns(),
        opts.output.display()
    );
    Ok(())
}
