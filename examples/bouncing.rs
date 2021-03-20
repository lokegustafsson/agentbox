use agentbox::{models::BouncingBalls, Status};
use std::{thread, time::Duration};

fn main() {
    env_logger::init();

    agentbox::run_with::<BouncingBalls, _>(Status::VISUAL, move |_world, _signals, _status| {
        thread::sleep(Duration::from_secs_f32(0.01));
    })
}
