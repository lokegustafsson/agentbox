//! Premade simulation environments. Pick one to run!

mod bouncing_balls;
mod inverted_double_pendulum;
mod simple;

pub use bouncing_balls::BouncingBalls;
pub use inverted_double_pendulum::InvertedDoublePendulum;
pub use simple::SimpleModel;
