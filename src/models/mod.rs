mod inverted_double_pendulum;
mod simple;

use crate::common::Solid;

pub use inverted_double_pendulum::InvertedDoublePendulum;
pub use simple::SimpleModel;

#[derive(Clone, Copy)]
pub struct Status {
    pub display_visual: bool,
    pub should_quit: bool,
    pub delta_time: f32,
    pub physics_substeps: u32,
}

impl Status {
    pub const VISUAL: Status = Status {
        display_visual: true,
        ..Status::HEADLESS
    };
    pub const HEADLESS: Status = Status {
        display_visual: false,
        should_quit: false,
        delta_time: 0.1,
        physics_substeps: 50,
    };
}

pub trait Model {
    type World: Send + Sync + Clone;
    type Signals;

    fn new_world() -> Self::World;
    fn new_signals() -> Self::Signals;

    fn update(world: &mut Self::World, signals: &Self::Signals);

    fn get_solids(world: &Self::World) -> Vec<Solid>;
}
