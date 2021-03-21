use cgmath::Matrix4;
use winit::event::KeyboardInput;

mod aircraft;
mod fps;

pub use aircraft::AircraftCamera;
pub use fps::FpsCamera;

pub trait Camera {
    fn new() -> Self;
    fn update(&mut self, delta_pos: f32);
    fn key_input(&mut self, key: KeyboardInput);
    fn mouse_input(&mut self, x: f64, y: f64, w: u32, h: u32);
    fn camera_to_world(&mut self) -> Matrix4<f32>;
}

// Often two keys are opposites. If neither or both are pressed nothing happens,
// but if one is pressed we go, say, either forwards of backwards. [`Choice`]
// reduces the boilerplate necesary in such a situation.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Choice {
    Neither,
    Go,
    Reverse,
    Both,
}

impl Choice {
    fn f32(self) -> f32 {
        use Choice::*;
        match self {
            Reverse => -1.0,
            Neither | Both => 0.0,
            Go => 1.0,
        }
    }
    fn go(&mut self, take: bool) {
        use Choice::*;
        *self = match self {
            Neither | Go if take => Go,
            Neither | Go if !take => Neither,
            Reverse | Both if take => Both,
            Reverse | Both if !take => Reverse,
            _ => unreachable!(),
        }
    }
    fn reverse(&mut self, take: bool) {
        use Choice::*;
        *self = match self {
            Neither | Reverse if take => Reverse,
            Neither | Reverse if !take => Neither,
            Go | Both if take => Both,
            Go | Both if !take => Go,
            _ => unreachable!(),
        }
    }
}
