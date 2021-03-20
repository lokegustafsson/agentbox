//! Premade simulation environments. Pick one to run!

mod inverted_double_pendulum;
mod simple;

pub use inverted_double_pendulum::InvertedDoublePendulum;
pub use simple::SimpleModel;
