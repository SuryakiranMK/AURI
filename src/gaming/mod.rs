// Project AURI: Gaming Mode & Performance Subsystem
// Zero-overhead game detection, working set RAM trimmer, and UI suspension.

pub mod detector;
pub mod trimmer;

pub use detector::{GameDetector, GameEvent};
pub use trimmer::{get_working_set_size_bytes, trim_working_set, WorkingSetReport};

