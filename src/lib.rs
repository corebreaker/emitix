//! `emitix` - A thread-safe event management library
mod traits;

pub mod event_hub;

pub use traits::{EventEmitter, EventManager};
