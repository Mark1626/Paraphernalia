use std::ffi::{CStr, c_void};
use std::marker::PhantomData;
use std::ptr;

use crate::ffi::*;
use crate::{Error, cstring, ensure_init};

/// A freshly-built record. Owns its `rec_record_t` until passed to
/// [`crate::Rset::append_record`], at which point ownership is transferred to
/// the containing record set.
pub struct Record {
    ptr: rec_record_t,
}

impl Record {
    pub fn new() -> Self {
        ensure_init();
        let ptr = unsafe { rec_record_new() };
        assert!(!ptr.is_null(), "rec_record_new returned NULL");
        Record { ptr }
    }

    pub fn append_field(&mut self, name: &str, value: &str) -> Result<(), Error> {
        let cn = cstring(name, "field name")?;
        let cv = cstring(value, "field value")?;
        unsafe {
            let field = rec_field_new(cn.as_ptr(), cv.as_ptr());
            if field.is_null() {
                return Err(Error::new("rec_field_new returned NULL"));
            }
            let mset = rec_record_mset(self.ptr);
            let elem = rec_mset_append(
                mset,
                MSET_FIELD as rec_mset_type_t,
                field as *mut c_void,
                MSET_FIELD as rec_mset_type_t,
            );
            if elem.is_null() {
                rec_field_destroy(field);
                return Err(Error::new("rec_mset_append (field) failed"));
            }
        }
        Ok(())
    }

    pub(crate) fn into_raw(self) -> rec_record_t {
        let ptr = self.ptr;
        std::mem::forget(self);
        ptr
    }
}

impl Default for Record {
    fn default() -> Self {
        Record::new()
    }
}

impl Drop for Record {
    fn drop(&mut self) {
        unsafe { rec_record_destroy(self.ptr) }
    }
}

pub struct RecordRef<'a> {
    ptr: rec_record_t,
    _marker: PhantomData<&'a ()>,
}

impl<'a> RecordRef<'a> {
    pub(crate) fn from_raw(ptr: rec_record_t) -> Self {
        RecordRef { ptr, _marker: PhantomData }
    }

    pub fn fields(&self) -> Fields<'_> {
        let iter = unsafe { rec_mset_iterator(rec_record_mset(self.ptr)) };
        Fields { iter, done: false, _marker: PhantomData }
    }

    pub fn as_ptr(&self) -> rec_record_t {
        self.ptr
    }

    /// Set the value of the first field with the given `name`. Returns `true`
    /// if such a field was found and updated, `false` otherwise.
    pub fn set_field(&mut self, name: &str, value: &str) -> Result<bool, Error> {
        let cn = cstring(name, "field name")?;
        let cv = cstring(value, "field value")?;
        unsafe {
            let field = rec_record_get_field_by_name(self.ptr, cn.as_ptr(), 0);
            if field.is_null() {
                return Ok(false);
            }
            if !rec_field_set_value(field, cv.as_ptr()) {
                return Err(Error::new("rec_field_set_value failed"));
            }
        }
        Ok(true)
    }
}

pub struct Fields<'a> {
    iter: rec_mset_iterator_t,
    done: bool,
    _marker: PhantomData<&'a RecordRef<'a>>,
}

impl<'a> Iterator for Fields<'a> {
    type Item = FieldRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let mut data: *const c_void = ptr::null();
        let advanced = unsafe {
            rec_mset_iterator_next(
                &mut self.iter,
                MSET_FIELD as rec_mset_type_t,
                &mut data,
                ptr::null_mut(),
            )
        };
        if !advanced {
            self.done = true;
            return None;
        }
        Some(FieldRef { ptr: data as rec_field_t, _marker: PhantomData })
    }
}

impl<'a> Drop for Fields<'a> {
    fn drop(&mut self) {
        unsafe { rec_mset_iterator_free(&mut self.iter) }
    }
}

pub struct FieldRef<'a> {
    ptr: rec_field_t,
    _marker: PhantomData<&'a ()>,
}

impl<'a> FieldRef<'a> {
    pub fn name(&self) -> String {
        unsafe { CStr::from_ptr(rec_field_name(self.ptr)).to_string_lossy().into_owned() }
    }

    pub fn value(&self) -> String {
        unsafe { CStr::from_ptr(rec_field_value(self.ptr)).to_string_lossy().into_owned() }
    }
}
