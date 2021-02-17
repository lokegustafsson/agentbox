mod physics;
mod solid_primitives;
mod visual;

use std::{
    sync::{atomic::AtomicUsize, Arc, Mutex},
    thread,
};
use winit::{event_loop::EventLoop, window::WindowBuilder};

pub use physics::{ControlSignals, WorldState};

const WHOLECANVAS_VERTEX: &[u8] = include_bytes!("../target/shaders/wholecanvas.vert.spv");
const SOLIDS_FRAGMENT: &[u8] = include_bytes!("../target/shaders/solids.frag.spv");

#[derive(Clone, Copy)]
pub struct Status {
    pub display_visual: bool,
    pub should_quit: bool,
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

pub(crate) struct WorldChannel {
    world: Mutex<Arc<WorldState>>,
    version: AtomicUsize,
}

impl WorldChannel {
    pub fn new() -> Self {
        Self {
            world: Mutex::new(Arc::new(WorldState::default())),
            version: AtomicUsize::new(0),
        }
    }
}

#[derive(Debug)]
pub(crate) enum SimulationEvent {
    RequestExit,
    RequestHide,
    RequestShow,
    SimulationPanic,
}

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
        thread::spawn(move || physics::run_physics(channel, proxy, controller, initial_status));
    }
    visual::run_event_loop(event_loop, window, channel, initial_status.display_visual)
}

#[cfg(test)]
mod test {
    use super::{run_with, Status};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_just_quit() {
        init();

        run_with(Status::HEADLESS, |_state, _signals, status| {
            status.should_quit = true;
        })
    }

    #[test]
    fn test_1000_float() {
        init();

        let mut i = 0;
        run_with(Status::HEADLESS, move |_state, signals, status| {
            status.should_quit = i == 1000;
            i += 1;
            signals.float = true;
        })
    }
}
