use std::{fs::File, sync::Arc};

use arrow::array::{ArrayRef, StringArray};
use arrow::record_batch::RecordBatch;
use parquet::arrow::{ArrowWriter, arrow_reader::ParquetRecordBatchReaderBuilder};
use parquet::column::page::Page;
use parquet::data_type::{ByteArray, ByteArrayType};
use parquet::encodings::decoding::{Decoder, PlainDecoder};
use parquet::file::properties::WriterProperties;
use parquet::file::reader::{FileReader, SerializedFileReader};

fn main() -> anyhow::Result<()> {
    // --- Write ---
    let values: ArrayRef = Arc::new(StringArray::from(vec!["a", "b", "a", "c", "b", "a"]));
    let batch = RecordBatch::try_from_iter([("value", values)])?;

    let props = WriterProperties::builder()
        .set_dictionary_enabled(true) // dictionary page + RLE_DICTIONARY-encoded data pages
        .build();
    let file = File::create("data.parquet")?;
    let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(props))?;
    writer.write(&batch)?;
    writer.close()?;

    // --- Read ---
    let file = File::open("data.parquet")?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;

    // Confirm the column chunk actually used RLE_DICTIONARY
    let col_meta = builder.metadata().row_group(0).column(0);
    println!("encodings: {:?}", col_meta.encodings().collect::<Vec<_>>());

    // col_meta.

    // --- Read only the dictionary page ---
    // Metadata stores just its offset; the page reader is lazy, so taking only
    // the first page reads the dictionary page and never touches data pages.
    println!("dictionary_page_offset: {:?}", col_meta.dictionary_page_offset());
    let reader = SerializedFileReader::new(File::open("data.parquet")?)?;
    let mut pages = reader.get_row_group(0)?.get_column_page_reader(0)?;
    if let Some(Page::DictionaryPage { buf, num_values, encoding, .. }) = pages.get_next_page()? {
        println!("dictionary page: {num_values} values, encoding {encoding:?}");
        // PLAIN encoding for byte arrays: repeated [u32 LE length][bytes]
        let mut data = &buf[..];
        for _ in 0..num_values {
            let len = u32::from_le_bytes(data[..4].try_into()?) as usize;
            println!("  {:?}", std::str::from_utf8(&data[4..4 + len])?);
            data = &data[4 + len..];
        }

        // Same page, decoded with parquet's own PlainDecoder (feature = "experimental")
        let mut decoder = PlainDecoder::<ByteArrayType>::new(0);
        decoder.set_data(buf.clone(), num_values as usize)?;
        let mut values = vec![ByteArray::new(); num_values as usize];
        decoder.get(&mut values)?;
        println!("via PlainDecoder: {:?}", values.iter().map(|v| v.as_utf8()).collect::<Result<Vec<_>, _>>()?);
    }

    for batch in builder.build()? {
        let batch = batch?;
        println!("read: {:?}", batch.column(0));
    }
    Ok(())
}
