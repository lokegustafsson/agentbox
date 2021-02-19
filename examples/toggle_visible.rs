use combat::{self, Status};
use std::time::{Duration, Instant};

fn main() {
    env_logger::init();

    let mut toggle_instant = Instant::now();

    combat::run_with(Status::VISUAL, move |_state, signals, status| {
        signals.float = true;

        if Instant::now().duration_since(toggle_instant) > Duration::from_secs(5) {
            toggle_instant = Instant::now();
            status.display_visual = !status.display_visual;
        }
    })
}
