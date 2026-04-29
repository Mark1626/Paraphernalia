use std::fs;

use recutils_rs::{Db, Record};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .expect("usage: append_record <file.rec> <Type> Field=Value [Field=Value ...]");
    let rset_type = args.next().expect("missing record type");
    let pairs: Vec<(String, String)> = args
        .map(|a| {
            let (k, v) = a.split_once('=').expect("expected Field=Value");
            (k.to_string(), v.to_string())
        })
        .collect();
    assert!(!pairs.is_empty(), "need at least one Field=Value");

    let text = fs::read_to_string(&path).expect("read file");
    let mut db = Db::parse_str(&text).expect("parse");

    let mut record = Record::new();
    for (k, v) in &pairs {
        record.append_field(k, v).expect("append field");
    }

    {
        let mut rset = db
            .rset_by_type(&rset_type)
            .unwrap_or_else(|| panic!("no record set of type {rset_type:?}"));
        rset.append_record(record).expect("append record");
    }

    let serialized = db.to_rec_string().expect("serialize");
    fs::write(&path, serialized).expect("write file");
    println!("appended 1 record to rset {rset_type:?} in {path}");
}
