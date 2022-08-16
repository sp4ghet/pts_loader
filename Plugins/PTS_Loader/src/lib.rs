pub mod clib;
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
