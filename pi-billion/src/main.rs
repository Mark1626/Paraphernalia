use std::fs::File;
use std::sync::Arc;

use arrow::array::{Int64Array, StringArray};
use arrow::buffer::{Buffer, OffsetBuffer, ScalarBuffer};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use memmap2::Mmap;
use parquet::arrow::ArrowWriter;
use parquet::basic::{Compression, Encoding};
use parquet::file::properties::WriterProperties;

const WINDOW: usize = 9;
/// Rows built per Arrow RecordBatch before handing off to the Parquet writer.
const BATCH_ROWS: usize = 1 << 20; // ~1M rows -> ~13MB working buffers

struct Args {
    input: String,
    output: String,
    limit: Option<usize>,
}

fn parse_args() -> Args {
    let mut input = "pi-billion.txt".to_string();
    let mut output = "pi-windows.parquet".to_string();
    let mut limit = None;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--input" | "-i" => input = it.next().expect("--input needs a value"),
            "--output" | "-o" => output = it.next().expect("--output needs a value"),
            "--limit" | "-n" => {
                limit = Some(
                    it.next()
                        .expect("--limit needs a value")
                        .parse()
                        .expect("--limit must be an integer"),
                )
            }
            other => panic!("unknown arg: {other}"),
        }
    }
    Args { input, output, limit }
}

fn main() {
    let args = parse_args();

    let file = File::open(&args.input).expect("open input");
    // SAFETY: file is read-only for the lifetime of the mapping.
    let mmap = unsafe { Mmap::map(&file).expect("mmap input") };
    let bytes = &mmap[..];

    // The file is "3." followed by the fractional digits of PI.
    assert!(bytes.len() >= 2 && &bytes[0..2] == b"3.", "expected file to start with '3.'");
    let digits = &bytes[2..];
    // Trim any trailing newline / whitespace just in case.
    let digits = {
        let mut end = digits.len();
        while end > 0 && !digits[end - 1].is_ascii_digit() {
            end -= 1;
        }
        &digits[..end]
    };
    assert!(digits.len() >= WINDOW, "need at least {WINDOW} digits");

    let total_windows = digits.len() - WINDOW + 1;
    let n_windows = match args.limit {
        Some(l) => total_windows.min(l),
        None => total_windows,
    };
    println!(
        "digits after decimal: {}, emitting {} windows of width {}",
        digits.len(),
        n_windows,
        WINDOW
    );

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("window", DataType::Utf8, false),
    ]));

    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        // Windows are nearly all distinct, so a dictionary only adds overhead.
        .set_dictionary_enabled(false)
        .set_encoding(Encoding::PLAIN)
        .set_max_row_group_size(BATCH_ROWS)
        .build();

    let out = File::create(&args.output).expect("create output");
    let mut writer = ArrowWriter::try_new(out, schema.clone(), Some(props)).expect("arrow writer");

    let start = std::time::Instant::now();
    let mut done = 0usize;
    while done < n_windows {
        let rows = BATCH_ROWS.min(n_windows - done);
        let batch = build_batch(digits, done, rows, &schema);
        writer.write(&batch).expect("write batch");
        done += rows;
        if done % (BATCH_ROWS * 64) == 0 || done == n_windows {
            let secs = start.elapsed().as_secs_f64();
            println!(
                "  {:>12}/{} rows ({:.1}%) {:.1}M rows/s",
                done,
                n_windows,
                100.0 * done as f64 / n_windows as f64,
                done as f64 / secs / 1e6
            );
        }
    }

    writer.close().expect("close writer");
    println!("done in {:.1}s -> {}", start.elapsed().as_secs_f64(), args.output);
}

/// Build a RecordBatch of `rows` windows starting at digit index `start`.
fn build_batch(digits: &[u8], start: usize, rows: usize, schema: &Arc<Schema>) -> RecordBatch {
    // Values are stored back-to-back: row i occupies bytes [i*9 .. i*9+9].
    let mut values = vec![0u8; rows * WINDOW];
    for i in 0..rows {
        let src = start + i;
        values[i * WINDOW..i * WINDOW + WINDOW].copy_from_slice(&digits[src..src + WINDOW]);
    }

    // Offsets: 0, 9, 18, ..., rows*9.
    let offsets: Vec<i32> = (0..=rows).map(|i| (i * WINDOW) as i32).collect();

    let offsets = OffsetBuffer::new(ScalarBuffer::from(offsets));
    let values = Buffer::from_vec(values);
    // SAFETY: every byte is an ASCII digit, hence valid UTF-8, and offsets are
    // monotonically increasing and in-bounds of `values`.
    let windows = unsafe { StringArray::new_unchecked(offsets, values, None) };

    // 1-based id matching each window's position in the digit stream.
    let ids = Int64Array::from_iter_values((0..rows).map(|i| (start + i + 1) as i64));

    RecordBatch::try_new(schema.clone(), vec![Arc::new(ids), Arc::new(windows)])
        .expect("record batch")
}
