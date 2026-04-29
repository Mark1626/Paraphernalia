use std::fs;

use recutils_rs::{Db, SelectionExpression};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .expect("usage: query_records <file.rec> <Type> <selection-expression>");
    let rset_type = args.next().expect("missing record type");
    let expr = args.next().expect("missing selection expression");

    let text = fs::read_to_string(&path).expect("read file");
    let mut db = Db::parse_str(&text).expect("parse");

    let rset = db
        .rset_by_type(&rset_type)
        .unwrap_or_else(|| panic!("no record set of type {rset_type:?}"));
    let selection_expression =
        SelectionExpression::compile(&expr, false).expect("compile selection expression");

    let mut matches = 0usize;
    for record in rset.records().filter(|r| selection_expression.matches(r)) {
        matches += 1;
        for field in record.fields() {
            println!("{}: {}", field.name(), field.value());
        }
        println!();
    }
    eprintln!("{matches} match(es)");
}
