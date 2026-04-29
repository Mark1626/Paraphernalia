use std::fs;

use recutils_rs::Db;

fn main() {
    let path = std::env::args().nth(1).expect("usage: count_records <file.rec>");
    let text = fs::read_to_string(&path).expect("read file");

    let mut db = Db::parse_str(&text).expect("parse");
    let n_rsets = db.num_rsets();
    let total: usize = (0..n_rsets)
        .map(|i| db.rset_at(i).expect("rset by index").num_records())
        .sum();
    println!("{}: {} records across {} record set(s)", path, total, n_rsets);
}
