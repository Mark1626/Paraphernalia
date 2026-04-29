use std::fs;
use std::process::ExitCode;

use recutils_rs::{Db, Sex};

fn usage() -> ! {
    eprintln!(
        "usage:\n  \
         modify_records <file.rec> <Type> <expr> set <Field>=<Value>\n  \
         modify_records <file.rec> <Type> <expr> delete"
    );
    std::process::exit(2);
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap_or_else(|| usage());
    let rset_type = args.next().unwrap_or_else(|| usage());
    let expr = args.next().unwrap_or_else(|| usage());
    let action = args.next().unwrap_or_else(|| usage());

    let text = fs::read_to_string(&path).expect("read file");
    let mut db = Db::parse_str(&text).expect("parse");
    let sex = Sex::compile(&expr, false).expect("compile selection expression");

    let summary = {
        let mut rset = db
            .rset_by_type(&rset_type)
            .unwrap_or_else(|| panic!("no record set of type {rset_type:?}"));

        match action.as_str() {
            "set" => {
                let pair = args.next().unwrap_or_else(|| usage());
                let (field, value) = pair
                    .split_once('=')
                    .unwrap_or_else(|| panic!("expected Field=Value, got {pair:?}"));
                let mut updated = 0usize;
                let mut missing = 0usize;
                for mut record in rset.records().filter(|r| sex.matches(r)) {
                    match record.set_field(field, value).expect("set field") {
                        true => updated += 1,
                        false => missing += 1,
                    }
                }
                format!(
                    "set {field}={value} on {updated} record(s); {missing} matching record(s) had no such field"
                )
            }
            "delete" => {
                let removed = rset.remove_matching(|r| sex.matches(r));
                format!("deleted {removed} record(s)")
            }
            other => {
                eprintln!("unknown action {other:?}");
                return ExitCode::from(2);
            }
        }
    };

    let serialized = db.to_rec_string().expect("serialize");
    fs::write(&path, serialized).expect("write file");
    println!("{summary}");
    ExitCode::SUCCESS
}
