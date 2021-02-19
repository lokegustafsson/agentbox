use combat::{self, Status};

fn main() {
    env_logger::init();

    combat::run_with(Status::VISUAL, move |_state, signals, _status| {
        signals.float = true;
    })
}
