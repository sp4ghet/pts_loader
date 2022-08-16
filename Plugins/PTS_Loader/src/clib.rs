use std::{ffi::CStr, os::raw::c_char};

use crate::*;

#[no_mangle]
pub extern "C" fn load_from_file(path: *const c_char, _buf: &mut *mut PtsPoint, _len: &mut isize) {
    let path = unsafe { CStr::from_ptr(path) }.to_str().unwrap();
    let mut points = match load::from_file(path) {
        Ok(p) => p,
        Err(_) => {
            *_len = -1;
            return;
        }
    };
    *_buf = points.as_mut_ptr();
    *_len = points.len() as _;
    std::mem::forget(points);
}

#[no_mangle]
pub extern "C" fn free_pts(buf: *mut PtsPoint, len: usize) {
    let data = unsafe { Vec::from_raw_parts(buf, len, len) }.to_owned();
    drop(data);
}
