#![deny(
    clippy::all,
    clippy::pedantic,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style
)]

mod common;
mod model;
mod simulation;
mod visual;

use common::{SimulationEvent, WorldChannel};
use std::{sync::Arc, thread};
use winit::{event_loop::EventLoop, window::WindowBuilder};

pub use model::{ControlSignals, Status, WorldState};

const WHOLECANVAS_VERTEX: &[u8] = include_bytes!("../target/shaders/wholecanvas.vert.spv");
const SOLIDS_FRAGMENT: &[u8] = include_bytes!("../target/shaders/solids.frag.spv");

pub fn run_with<F>(initial_status: Status, controller: F) -> !
where
    F: Send + 'static + FnMut(&WorldState, &mut ControlSignals, &mut Status),
{
    let event_loop: EventLoop<SimulationEvent> = EventLoop::with_user_event();
    let window = WindowBuilder::new()
        .with_title("Combat")
        .with_visible(initial_status.display_visual)
        .build(&event_loop)
        .unwrap();
    let channel = Arc::new(WorldChannel::new());

    {
        let channel = channel.clone();
        let proxy = event_loop.create_proxy();
        let initial_status = initial_status;
        thread::spawn(move || {
            simulation::run_simulation(channel, proxy, controller, initial_status)
        });
    }
    visual::run_event_loop(event_loop, window, channel, initial_status.display_visual)
}
