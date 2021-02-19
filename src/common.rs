use crate::models::Model;
use cgmath::Vector3;
use getset::CopyGetters;
use std::sync::{atomic::AtomicUsize, Arc, Mutex};

// Communication between the event loop and the simulation thread

pub struct WorldChannel<M: Model> {
    pub world: Mutex<Arc<M::World>>,
    pub version: AtomicUsize,
}

impl<M: Model> WorldChannel<M> {
    pub fn new() -> Self {
        Self {
            world: Mutex::new(Arc::new(M::new_world())),
            version: AtomicUsize::new(0),
        }
    }
}

#[derive(Debug)]
pub enum SimulationEvent {
    RequestExit,
    RequestHide,
    RequestShow,
    SimulationPanic,
}

// Structs that will be sent to the GPU

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

impl Cylinder {
    pub fn new(
        face_a: Vector3<f32>,
        face_b: Vector3<f32>,
        radius: f32,
        color: Vector3<f32>,
    ) -> Self {
        Self {
            face_a,
            _padding1: 0,
            face_b,
            radius,
            color,
            _padding2: 0,
        }
    }
}
