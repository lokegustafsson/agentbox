use agentbox::{models::SimpleModel, Status};
use cgmath::Vector3;
use std::{thread, time::Duration};

fn main() {
    env_logger::init();

    agentbox::run_with::<SimpleModel, _>(Status::VISUAL, move |world, signals, _status| {
        signals.accel = Vector3::unit_z() * 0.01;

        signals.target_color = if world.color.x > 0.6 {
            Vector3::unit_y()
        } else if world.color.y > 0.6 {
            Vector3::unit_z()
        } else if world.color.z > 0.6 {
            Vector3::unit_x()
        } else {
            signals.target_color
        };

        thread::sleep(Duration::from_secs_f32(0.01));
    })
}
