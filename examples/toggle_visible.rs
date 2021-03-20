use agentbox::{models::SimpleModel, Status};
use cgmath::{prelude::*, Vector3};
use std::time::{Duration, Instant};

fn main() {
    env_logger::init();

    let mut toggle_instant = Instant::now();

    agentbox::run_with::<SimpleModel, _>(Status::VISUAL, move |_state, signals, status| {
        signals.accel = Vector3::zero();

        if Instant::now().duration_since(toggle_instant) > Duration::from_secs(5) {
            toggle_instant = Instant::now();
            status.display_visual = !status.display_visual;
        }
    })
}
