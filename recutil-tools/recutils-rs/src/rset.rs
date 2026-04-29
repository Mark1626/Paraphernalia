use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr;

use crate::Error;
use crate::db::Db;
use crate::ffi::*;
use crate::record::{Record, RecordRef};

pub struct Rset<'a> {
    ptr: rec_rset_t,
    _marker: PhantomData<&'a mut Db>,
}

impl<'a> Rset<'a> {
    pub(crate) fn from_raw(ptr: rec_rset_t) -> Self {
        Rset { ptr, _marker: PhantomData }
    }

    pub fn num_records(&self) -> usize {
        unsafe { rec_rset_num_records(self.ptr) }
    }

    /// The record-descriptor record (everything declared in the `%rec:` /
    /// `%type:` / `%key:` / `%mandatory:` block). `None` for record sets that
    /// have no descriptor.
    pub fn descriptor(&self) -> Option<RecordRef<'_>> {
        let p = unsafe { rec_rset_descriptor(self.ptr) };
        (!p.is_null()).then(|| RecordRef::from_raw(p))
    }

    pub fn records(&self) -> Records<'_> {
        let iter = unsafe { rec_mset_iterator(rec_rset_mset(self.ptr)) };
        Records { iter, done: false, _marker: PhantomData }
    }

    /// Remove every record for which `pred` returns true. Returns the count
    /// of records removed.
    pub fn remove_matching<F>(&mut self, mut pred: F) -> usize
    where
        F: FnMut(&RecordRef<'_>) -> bool,
    {
        let mset = unsafe { rec_rset_mset(self.ptr) };
        let mut to_remove: Vec<usize> = Vec::new();
        let mut iter = unsafe { rec_mset_iterator(mset) };
        let mut idx: usize = 0;
        loop {
            let mut data: *const c_void = ptr::null();
            let advanced = unsafe {
                rec_mset_iterator_next(
                    &mut iter,
                    MSET_RECORD as rec_mset_type_t,
                    &mut data,
                    ptr::null_mut(),
                )
            };
            if !advanced {
                break;
            }
            let r = RecordRef::from_raw(data as rec_record_t);
            if pred(&r) {
                to_remove.push(idx);
            }
            idx += 1;
        }
        unsafe { rec_mset_iterator_free(&mut iter) };

        let n = to_remove.len();
        for pos in to_remove.into_iter().rev() {
            unsafe { rec_mset_remove_at(mset, MSET_RECORD as rec_mset_type_t, pos) };
        }
        n
    }

    pub fn append_record(&mut self, record: Record) -> Result<(), Error> {
        let raw = record.into_raw();
        let elem = unsafe {
            rec_mset_append(
                rec_rset_mset(self.ptr),
                MSET_RECORD as rec_mset_type_t,
                raw as *mut c_void,
                MSET_RECORD as rec_mset_type_t,
            )
        };
        if elem.is_null() {
            unsafe { rec_record_destroy(raw) };
            return Err(Error::new("rec_mset_append (record) failed"));
        }
        Ok(())
    }
}

pub struct Records<'a> {
    iter: rec_mset_iterator_t,
    done: bool,
    _marker: PhantomData<&'a Rset<'a>>,
}

impl<'a> Iterator for Records<'a> {
    type Item = RecordRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let mut data: *const c_void = ptr::null();
        let advanced = unsafe {
            rec_mset_iterator_next(
                &mut self.iter,
                MSET_RECORD as rec_mset_type_t,
                &mut data,
                ptr::null_mut(),
            )
        };
        if !advanced {
            self.done = true;
            return None;
        }
        Some(RecordRef::from_raw(data as rec_record_t))
    }
}

impl<'a> Drop for Records<'a> {
    fn drop(&mut self) {
        unsafe { rec_mset_iterator_free(&mut self.iter) }
    }
}
