mod bounding_ball_tree;
mod camera;
mod graphics;
mod solids_renderer;

use crate::{
    common::{SimulationEvent, WorldChannel},
    model::WorldState,
};
use anyhow::Result;
use async_std::task::block_on;
use camera::Camera;
use graphics::Graphics;
use std::{
    sync::{atomic::Ordering, Arc, MutexGuard},
    time::{Duration, Instant},
};
use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub(crate) fn run_event_loop(
    event_loop: EventLoop<SimulationEvent>,
    window: Window,
    channel: Arc<WorldChannel>,
    initial_visible: bool,
) -> ! {
    let mut last_version = channel.version.load(Ordering::SeqCst);
    let mut graphics = block_on(Graphics::initialize(&window)).unwrap();
    let mut camera = Camera::new();
    let mut last_timestamp = Instant::now();
    let mut capture_mouse = false;
    let mut slow_mode = false;
    let mut visible = initial_visible;

    let frame_time = Duration::from_secs_f32(1.0 / 60.0);

    event_loop.run(move |event, _, control_flow| {
        if visible {
            // Limit to 60 fps
            *control_flow = ControlFlow::WaitUntil(Instant::now() + frame_time);
        } else {
            // Await an event
            *control_flow = ControlFlow::Wait;
        }
        match event {
            // Update
            Event::MainEventsCleared => {
                let now_timestamp = Instant::now();
                if visible {
                    camera.update(now_timestamp.duration_since(last_timestamp));
                    window.request_redraw();
                }
                last_timestamp = now_timestamp;
            }
            // Render
            Event::RedrawRequested(_window_id) => {
                // Skip [update_world] if [world] is unchanged
                let newest = channel.version.load(Ordering::SeqCst);
                if newest != last_version {
                    last_version = newest;
                    let world: &WorldState = {
                        let guard: MutexGuard<'_, Arc<WorldState>> = channel.world.lock().unwrap();
                        &guard.clone()
                    };
                    graphics.update_world(world);
                }
                graphics.render(camera.camera_to_world())
            }
            // Handle window event
            Event::WindowEvent { event: w_event, .. } => match w_event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => graphics.resize(new_size),
                WindowEvent::ModifiersChanged(mods) => {
                    if mods.alt() || mods.logo() {
                        stop_capture_mouse(&window);
                        capture_mouse = false;
                    } else {
                        capture_mouse = begin_capture_mouse(&window).is_ok();
                    }
                    slow_mode = mods.ctrl();
                }
                WindowEvent::KeyboardInput { input: key, .. } => camera.key_input(key, slow_mode),
                WindowEvent::CursorMoved { position: pos, .. } => {
                    if capture_mouse && continue_capture_mouse(&window) {
                        let size = window.inner_size();
                        camera.mouse_input(pos.x, pos.y, size.width, size.height);
                    }
                }
                WindowEvent::Focused(true) => capture_mouse = begin_capture_mouse(&window).is_ok(),
                WindowEvent::Focused(false) => {
                    stop_capture_mouse(&window);
                    capture_mouse = false;
                }
                _ => {}
            },
            Event::UserEvent(user_event) => match user_event {
                SimulationEvent::RequestExit => *control_flow = ControlFlow::Exit,
                SimulationEvent::SimulationPanic => *control_flow = ControlFlow::Exit,
                SimulationEvent::RequestHide => {
                    visible = false;
                    window.set_visible(false);
                    stop_capture_mouse(&window);
                    *control_flow = ControlFlow::Wait;
                }
                SimulationEvent::RequestShow => {
                    visible = true;
                    window.set_visible(true);
                    *control_flow = ControlFlow::WaitUntil(Instant::now() + frame_time);
                }
            },
            _ => {}
        }
    })
}

fn begin_capture_mouse(window: &Window) -> Result<()> {
    window.set_cursor_grab(true)?;
    let size = window.inner_size();
    window.set_cursor_position(PhysicalPosition::new(size.width / 2, size.height / 2))?;
    window.set_cursor_visible(false);
    Ok(())
}
fn continue_capture_mouse(window: &Window) -> bool {
    let size = window.inner_size();
    window
        .set_cursor_position(PhysicalPosition::new(size.width / 2, size.height / 2))
        .is_ok()
}
fn stop_capture_mouse(window: &Window) {
    window.set_cursor_grab(false).unwrap();
    window.set_cursor_visible(true);
}
