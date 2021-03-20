use crate::{Model, Status};
use log::{error, warn};
use std::{
    panic,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};
use winit::event_loop::EventLoopProxy;

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

pub fn run_simulation<M: Model, F>(
    channel: Arc<WorldChannel<M>>,
    proxy: EventLoopProxy<SimulationEvent>,
    mut controller: F,
    initial_status: Status,
) where
    F: Send + FnMut(&M::World, &mut M::Signals, &mut Status),
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

    let mut world: M::World = {
        let arc: Arc<M::World> = channel.world.lock().unwrap().clone();
        (*arc).clone()
    };
    let mut signals = M::new_signals();
    let mut status = initial_status;
    let mut visible = false; // The event loop is initially not visible

    loop {
        controller(&world, &mut signals, &mut status);
        M::update(&mut world, &signals);

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
