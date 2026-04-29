use crate::ffi::*;
use crate::record::RecordRef;
use crate::{Error, cstring, ensure_init};

/// A compiled selection expression — equivalent to `recsel -e <expr>`.
pub struct Sex {
    ptr: rec_sex_t,
}

impl Sex {
    pub fn compile(expr: &str, case_insensitive: bool) -> Result<Self, Error> {
        ensure_init();
        let c_expr = cstring(expr, "selection expression")?;
        unsafe {
            let ptr = rec_sex_new(case_insensitive);
            if ptr.is_null() {
                return Err(Error::new("rec_sex_new returned NULL"));
            }
            if !rec_sex_compile(ptr, c_expr.as_ptr()) {
                rec_sex_destroy(ptr);
                return Err(Error::new(format!("failed to compile expression {expr:?}")));
            }
            Ok(Sex { ptr })
        }
    }

    pub fn matches(&self, record: &RecordRef<'_>) -> bool {
        let mut status = false;
        let ok = unsafe { rec_sex_eval(self.ptr, record.as_ptr(), &mut status) };
        ok && status
    }
}

impl Drop for Sex {
    fn drop(&mut self) {
        unsafe { rec_sex_destroy(self.ptr) }
    }
}
