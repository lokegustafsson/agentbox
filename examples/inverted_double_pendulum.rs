use agentbox::{models::InvertedDoublePendulum, Status};
use cgmath::Vector2;
use std::{thread, time::Duration};

fn main() {
    env_logger::init();
    const WIGGLE: u32 = 0;

    let mut i = 0;
    agentbox::run_with::<InvertedDoublePendulum, _>(
        Status::VISUAL,
        move |world, signals, _status| {
            if i < WIGGLE {
                i += 1;
                signals.base_accel = Vector2::unit_x();
            } else {
                signals.base_accel = -world.base_pos
            }

            thread::sleep(Duration::from_secs_f32(0.01));
        },
    )
}
