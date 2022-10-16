//! Agentbox is a framework for building and using simulation environments for
//! autonomous control. You will need
//!
//! - An environment: A zero-sized type `M` implementing the [`Model`] trait.
//! - A controller: An `FnMut(&M::World, &mut M::Signals, &mut Status)`.
//!
//! with which to call [`run_with`] to run the simulation. [`models`] contain
//! several premade models, but you will need to bring your own controller -
//! as a closure or as a function.
//!
//! A tiny example:
//! ```
//! use agentbox::{Status, models::InvertedDoublePendulum as IDP};
//! use std::{thread, time::Duration};
//!
//! fn main() {
//!     env_logger::init();
//!     agentbox::run_with::<IDP, _>(Status::VISUAL, |_world, _signals, _status| {
//!         thread::sleep(Duration::from_secs_f32(0.01));
//!     })
//! }
//! ```

#![deny(
    clippy::all,
    clippy::pedantic,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style
)]

pub mod models;
pub mod physics;
pub use solid::Solid;

mod run; // The simulation thread loop
mod solid; // The Solid type
mod visual; // Everything graphical

use run::{SimulationEvent, WorldChannel};
use std::{mem, sync::Arc, thread};
use winit::{
    event_loop::{EventLoop, EventLoopBuilder},
    window::WindowBuilder,
};

// Compiled shaders are referenced here, but used elsewhere. This allows us to
// restructure the repo without breaking things.
const WHOLECANVAS_VERTEX: &[u8] = include_bytes!("../target/shaders/wholecanvas.vert.spv");
const SOLIDS_FRAGMENT: &[u8] = include_bytes!("../target/shaders/solids.frag.spv");

/// Call this to run the simulation.
pub fn run_with<M: Model + 'static, F>(initial_status: Status, controller: F) -> !
where
    F: Send + 'static + FnMut(&M::World, &mut M::Signals, &mut Status),
{
    assert_eq!(
        0,
        mem::size_of::<M>(),
        "A non-zero-sized Model type is nonsensical.",
    );
    let event_loop: EventLoop<SimulationEvent> = EventLoopBuilder::with_user_event().build();
    let window = WindowBuilder::new()
        .with_title("Agentbox")
        .with_visible(initial_status.display_visual)
        .build(&event_loop)
        .unwrap();
    let channel = Arc::new(WorldChannel::<M>::new());

    {
        let channel = channel.clone();
        let proxy = event_loop.create_proxy();
        let initial_status = initial_status;
        thread::spawn(move || run::run_simulation(channel, proxy, controller, initial_status));
    }
    visual::run_event_loop(event_loop, window, channel, initial_status.display_visual)
}

/// Controller-simulation tick-to-tick communication.
///
/// Read and write to these fields in your controller, and the simulation will
/// respond before next tick.
#[derive(Clone, Copy)]
pub struct Status {
    pub display_visual: bool,
    pub should_quit: bool,
}

impl Status {
    pub const VISUAL: Status = Status {
        display_visual: true,
        ..Status::HEADLESS
    };
    pub const HEADLESS: Status = Status {
        display_visual: false,
        should_quit: false,
    };
}

/// A model defines the environment in which to run the simulation. You may use
/// any model from [crate::models], or build your own by implementing this
/// trait.
pub trait Model {
    type World: Send + Sync + Clone;
    type Signals;

    fn new_world() -> Self::World;
    fn new_signals() -> Self::Signals;

    fn update(world: &mut Self::World, signals: &Self::Signals);

    fn get_solids(world: &Self::World) -> Vec<Solid>;
}
