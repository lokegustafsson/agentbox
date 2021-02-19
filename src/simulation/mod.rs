use crate::{
    common::{SimulationEvent, WorldChannel},
    model::{ControlSignals, Status, WorldState},
};
use cgmath::Vector3;
use log::{error, warn};
use std::{
    panic,
    sync::{atomic::Ordering, Arc, Mutex},
};
use winit::event_loop::EventLoopProxy;

// Contains physics accelerator structures
struct Simulation;

impl Simulation {
    pub fn new(state: &WorldState) -> Self {
        let _ = state;
        Self
    }
    pub fn update(&mut self, state: &mut WorldState, signals: &ControlSignals) {
        if signals.float {
            state.pos += Vector3::unit_z() * 0.0;
        }
    }
}

pub(crate) fn run_simulation<F>(
    channel: Arc<WorldChannel>,
    proxy: EventLoopProxy<SimulationEvent>,
    mut controller: F,
    initial_status: Status,
) where
    F: Send + FnMut(&WorldState, &mut ControlSignals, &mut Status),
{
    {
        let proxy = Mutex::new(proxy.clone());
        panic::set_hook(Box::new(move |panic_info| {
            let line = panic_info.location().unwrap();
            if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                error!("panic in simulation thread: {:?}. {}", s, line);
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                error!("panic in simulation thread: {:?}. {}", s, line);
            } else {
                error!("unprintable panic in GUI thread at {}", line);
            }
            match proxy.lock() {
                Ok(p) => match p.send_event(SimulationEvent::SimulationPanic) {
                    Ok(()) => {}
                    Err(_loop_closed) => {
                        error!("Sim failed to inform main of its panic: main already exited")
                    }
                },
                Err(poison) => {
                    error!("Sim failed to inform main of its panic: {:?}", poison)
                }
            }
        }));
    }

    let mut world: WorldState = {
        let arc: Arc<WorldState> = channel.world.lock().unwrap().clone();
        (*arc).clone()
    };
    let mut signals = ControlSignals::default();
    let mut status = initial_status;
    let mut visible = false; // The event loop is initially not visible

    let mut simulation = Simulation::new(&world);

    loop {
        controller(&world, &mut signals, &mut status);
        simulation.update(&mut world, &signals);

        // Tell GUI to quit
        if status.should_quit {
            warn!("Simulation thread exiting.");
            proxy.send_event(SimulationEvent::RequestExit).unwrap();
        }

        // Toggle visibility
        if status.display_visual != visible {
            visible = status.display_visual;
            match visible {
                true => proxy.send_event(SimulationEvent::RequestShow).unwrap(),
                false => proxy.send_event(SimulationEvent::RequestHide).unwrap(),
            }
        }

        // Push world to GUI
        if visible {
            *channel.world.lock().unwrap() = Arc::new(world.clone());
            channel.version.fetch_add(1, Ordering::SeqCst);
        }
    }
}
