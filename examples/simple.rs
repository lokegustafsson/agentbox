use combat::{self, Status};
use log::info;
use cgmath::prelude::*;

fn main() {
    env_logger::init();

    info!("Running the simple example.");

    let mut i = 0;
    combat::run_with(Status::VISUAL, move |_state, signals, _status| {
        signals.float = true;
        i += 1;
        if i % 1000 == 0 {
            //println!("Tick: {}", i);
        }
    })
}
