use std::{ffi::CStr, os::raw::c_char};

pub mod load;

#[repr(C)]
#[derive(Eq, PartialEq, Clone, Hash, Debug, Copy)]
pub struct Vec3<N> {
    pub x: N,
    pub y: N,
    pub z: N,
}

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Copy)]
pub struct PtsPoint {
    pub point: Vec3<f32>,
    pub intensity: i32,
    pub rgb: Vec3<u8>,
}

#[repr(C)]
pub struct Buffer {
    pub buf: *mut PtsPoint,
    pub len: usize,
}

#[no_mangle]
pub extern "C" fn load_from_file(path: *const c_char, _buf: &mut *mut PtsPoint, _len: &mut usize) {
    let path = unsafe { CStr::from_ptr(path) }.to_str().unwrap();
    let mut points = load::from_file(path);
    *_buf = points.as_mut_ptr();
    *_len = points.len();
    std::mem::forget(points);
}

#[no_mangle]
pub extern "C" fn free_pts(buf: *mut PtsPoint, len: usize) {
    let data = unsafe { Vec::from_raw_parts(buf, len, len) }.to_owned();
    drop(data);
}
