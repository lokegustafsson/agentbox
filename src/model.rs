use crate::common::{Cuboid, Cylinder, Sphere};
use cgmath::Vector3;

#[derive(Clone, Copy)]
pub struct Status {
    pub display_visual: bool,
    pub should_quit: bool,
}

#[derive(Clone)]
pub struct WorldState {
    pub pos: Vector3<f32>,
}

pub struct ControlSignals {
    pub float: bool,
}

impl Status {
    pub const VISUAL: Status = Status {
        display_visual: true,
        should_quit: false,
    };
    pub const HEADLESS: Status = Status {
        display_visual: false,
        should_quit: false,
    };
}

impl WorldState {
    pub(crate) fn get_solids(&self) -> (Vec<Sphere>, Vec<Cylinder>, Vec<Cuboid>) {
        const COLOR: Vector3<f32> = Vector3::new(0.0, 0.3, 0.6);
        (
            vec![Sphere::new(self.pos, 1.0, COLOR); 5],
            Vec::new(),
            Vec::new(),
        )
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            pos: Vector3::new(0.0, 0.0, 3.0),
        }
    }
}

impl Default for ControlSignals {
    fn default() -> Self {
        Self { float: false }
    }
}
