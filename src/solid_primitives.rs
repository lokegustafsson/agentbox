use cgmath::Vector3;
use getset::CopyGetters;

#[repr(C)]
#[derive(Copy, Clone, Debug, CopyGetters)]
pub struct Sphere {
    #[getset(get_copy = "pub")]
    pos: Vector3<f32>,
    #[getset(get_copy = "pub")]
    radius: f32,
    color: Vector3<f32>,
    _padding: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, CopyGetters)]
pub struct Cylinder {
    #[getset(get_copy = "pub")]
    face_a: Vector3<f32>,
    _padding1: u32,

    #[getset(get_copy = "pub")]
    face_b: Vector3<f32>,
    #[getset(get_copy = "pub")]
    radius: f32,

    color: Vector3<f32>,
    _padding2: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, CopyGetters)]
pub struct Cuboid {
    #[getset(get_copy = "pub")]
    corner: Vector3<f32>,
    _padding1: u32,

    #[getset(get_copy = "pub")]
    axis_a: Vector3<f32>,
    _padding2: u32,

    #[getset(get_copy = "pub")]
    axis_b: Vector3<f32>,
    #[getset(get_copy = "pub")]
    width: f32,

    color: Vector3<f32>,
    _padding3: u32,
}

unsafe impl bytemuck::Pod for Sphere {}
unsafe impl bytemuck::Zeroable for Sphere {}

unsafe impl bytemuck::Pod for Cylinder {}
unsafe impl bytemuck::Zeroable for Cylinder {}

unsafe impl bytemuck::Pod for Cuboid {}
unsafe impl bytemuck::Zeroable for Cuboid {}

impl Sphere {
    pub fn new(pos: Vector3<f32>, radius: f32, color: Vector3<f32>) -> Self {
        Self {
            pos,
            radius,
            color,
            _padding: 0,
        }
    }
    pub fn is_valid(&self) -> bool {
        self.pos.x.is_finite()
            && self.pos.y.is_finite()
            && self.pos.z.is_finite()
            && self.radius.is_finite()
            && self.radius.is_sign_positive()
    }
}
