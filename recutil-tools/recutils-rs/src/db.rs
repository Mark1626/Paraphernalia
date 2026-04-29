use std::ffi::{CStr, c_char, c_void};
use std::ptr;

use crate::ffi::*;
use crate::rset::Rset;
use crate::{Error, cstring, ensure_init};

unsafe extern "C" {
    fn free(ptr: *mut c_void);
}

pub struct Db {
    ptr: rec_db_t,
}

impl Db {
    pub fn parse_str(text: &str) -> Result<Self, Error> {
        ensure_init();
        let c_text = cstring(text, "rec source")?;
        unsafe {
            let parser = rec_parser_new_str(c_text.as_ptr(), c"input".as_ptr());
            if parser.is_null() {
                return Err(Error::new("rec_parser_new_str returned NULL"));
            }
            let mut db: rec_db_t = ptr::null_mut();
            let ok = rec_parse_db(parser, &mut db);
            rec_parser_destroy(parser);
            if !ok || db.is_null() {
                if !db.is_null() {
                    rec_db_destroy(db);
                }
                return Err(Error::new("rec_parse_db failed"));
            }
            Ok(Db { ptr: db })
        }
    }

    pub fn num_rsets(&self) -> usize {
        unsafe { rec_db_size(self.ptr) }
    }

    pub fn rset_at(&mut self, idx: usize) -> Option<Rset<'_>> {
        let p = unsafe { rec_db_get_rset(self.ptr, idx) };
        (!p.is_null()).then(|| Rset::from_raw(p))
    }

    pub fn rset_by_type(&mut self, name: &str) -> Option<Rset<'_>> {
        let c_name = cstring(name, "rset type").ok()?;
        let p = unsafe { rec_db_get_rset_by_type(self.ptr, c_name.as_ptr()) };
        (!p.is_null()).then(|| Rset::from_raw(p))
    }

    pub fn to_rec_string(&self) -> Result<String, Error> {
        unsafe {
            let mut buf: *mut c_char = ptr::null_mut();
            let mut size: usize = 0;
            let writer = rec_writer_new_str(&mut buf, &mut size);
            if writer.is_null() {
                return Err(Error::new("rec_writer_new_str returned NULL"));
            }
            let ok = rec_write_db(writer, self.ptr);
            rec_writer_destroy(writer);
            if !ok || buf.is_null() {
                if !buf.is_null() {
                    free(buf as *mut c_void);
                }
                return Err(Error::new("rec_write_db failed"));
            }
            let s = CStr::from_ptr(buf).to_string_lossy().into_owned();
            free(buf as *mut c_void);
            Ok(s)
        }
    }
}

impl Drop for Db {
    fn drop(&mut self) {
        unsafe { rec_db_destroy(self.ptr) }
    }
}
