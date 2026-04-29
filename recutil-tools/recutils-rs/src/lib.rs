//! Safe(r) wrapper around GNU recutils' `librec`.
//!
//! The raw bindings are still available under [`ffi`] for anything the safe
//! layer doesn't yet cover.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod ffi {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod db;
mod record;
mod rset;
mod selection_expression;

#[cfg(feature = "arrow")]
pub mod arrow;

pub use db::Db;
pub use record::{FieldRef, Fields, Record, RecordRef};
pub use rset::{Records, Rset};
pub use selection_expression::SelectionExpression;

use std::ffi::CString;
use std::fmt;
use std::sync::Once;

#[derive(Debug)]
pub struct Error(String);

impl Error {
    pub(crate) fn new(msg: impl Into<String>) -> Self {
        Error(msg.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

pub(crate) fn cstring(s: &str, what: &str) -> Result<CString, Error> {
    CString::new(s).map_err(|_| Error::new(format!("{what} contains an interior NUL byte")))
}

pub(crate) fn ensure_init() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| unsafe { ffi::rec_init() });
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
%rec: Book

Title: Refactoring
Author: Martin Fowler

Title: Domain-Driven Design
Author: Eric Evans
";

    #[test]
    fn parse_and_count() {
        let mut db = Db::parse_str(SAMPLE).unwrap();
        assert_eq!(db.num_rsets(), 1);
        let rset = db.rset_by_type("Book").unwrap();
        assert_eq!(rset.num_records(), 2);
    }

    #[test]
    fn selection_expression_filters() {
        let mut db = Db::parse_str(SAMPLE).unwrap();
        let rset = db.rset_by_type("Book").unwrap();
        let selection_expression = SelectionExpression::compile("Author = 'Eric Evans'", false).unwrap();
        let hits = rset.records().filter(|r| selection_expression.matches(r)).count();
        assert_eq!(hits, 1);
    }

    #[test]
    fn set_field_updates_matching() {
        let mut db = Db::parse_str(SAMPLE).unwrap();
        let selection_expression = SelectionExpression::compile("Author = 'Eric Evans'", false).unwrap();
        let rset = db.rset_by_type("Book").unwrap();
        let mut updated = 0;
        for mut r in rset.records().filter(|r| selection_expression.matches(r)) {
            assert!(r.set_field("Author", "Evans, Eric").unwrap());
            updated += 1;
        }
        assert_eq!(updated, 1);
        let s = db.to_rec_string().unwrap();
        assert!(s.contains("Evans, Eric"));
        assert!(!s.contains("Author: Eric Evans"));
    }

    #[test]
    fn remove_matching_drops_records() {
        let mut db = Db::parse_str(SAMPLE).unwrap();
        let selection_expression = SelectionExpression::compile("Author = 'Martin Fowler'", false).unwrap();
        let removed = {
            let mut rset = db.rset_by_type("Book").unwrap();
            rset.remove_matching(|r| selection_expression.matches(r))
        };
        assert_eq!(removed, 1);
        assert_eq!(db.rset_by_type("Book").unwrap().num_records(), 1);
    }

    #[test]
    fn append_round_trip() {
        let mut db = Db::parse_str(SAMPLE).unwrap();
        {
            let mut rset = db.rset_by_type("Book").unwrap();
            let mut rec = Record::new();
            rec.append_field("Title", "TDD").unwrap();
            rec.append_field("Author", "Kent Beck").unwrap();
            rset.append_record(rec).unwrap();
        }
        let serialized = db.to_rec_string().unwrap();
        assert!(serialized.contains("Kent Beck"));
        let mut db2 = Db::parse_str(&serialized).unwrap();
        assert_eq!(db2.rset_by_type("Book").unwrap().num_records(), 3);
    }
}
