//! `emitix` - A thread-safe event management library
mod traits;

pub mod event_hub;

#[cfg(feature = "leptos")]
pub mod leptos;

pub use traits::{EventEmitter, EventManager};
