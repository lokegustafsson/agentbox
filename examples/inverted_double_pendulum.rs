use cgmath::Vector2;
use combat::{
    self,
    models::{InvertedDoublePendulum, Status},
};
use std::{thread, time::Duration};

fn main() {
    env_logger::init();

    combat::run_with::<InvertedDoublePendulum, _>(
        Status::VISUAL,
        move |_world, signals, _status| {
            signals.bottom_accel = Vector2::unit_x() * 0.01;

            thread::sleep(Duration::from_secs_f32(0.01));
        },
    )
}
