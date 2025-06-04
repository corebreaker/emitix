//! `emitix` - A thread-safe event management library
//!
//! This module provides thread-safe event management through the `EventManager` struct.
//! `EventManager` allows adding event listeners and emitting events to all registered listeners.
//!
//! It is designed for use in multithreaded environments while ensuring thread safety using `Arc` and `Mutex`.
use anyhow::{Result, Error};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// `EventManager` is a thread-safe structure for managing events.
///
/// # Features
/// - Allows adding event listeners.
/// - Emits events to all registered listeners.
/// - Uses `Arc` and `Mutex` to ensure thread safety.
///
/// # Example Usage
/// ```rust
/// use emitix::EventManager;
///
/// let manager = EventManager::new();
///
/// manager
///     .add_listener("Events You Like", |event: String| {
///         println!("Event received: {}", event);
///     })
///     .unwrap();
///
/// manager
///     .emit("Events You Like", String::from("Test Event"))
///     .unwrap();
/// ```
///
/// # Thread-Safety Guarantees
/// - Listeners are stored in a `Mutex`, ensuring exclusive access.
/// - The structure can be cloned and shared across multiple threads using `Arc`.
/// - Listeners must be thread-safe functions (`Send` and `Sync`).
#[derive(Clone)]
pub struct EventManager<T: Clone + Send + Sync + 'static> {
    listeners: Arc<Mutex<HashMap<String, Vec<Box<dyn FnMut(T) + Send + Sync + 'static>>>>>,
}

impl<T: Clone + Send + Sync + 'static> EventManager<T> {
    /// Creates a new instance of `EventManager`.
    ///
    /// # Returns
    /// A new, empty instance of `EventManager` ready to register listeners.
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Lists all event kinds that have registered listeners.
    ///
    /// # Returns
    /// - `Ok(Vec<String>)` containing the names of all event kinds.
    /// - `Err(anyhow::Error)` if access to the `Mutex` failed.
    ///
    /// # Example
    /// ```rust
    /// let manager = emitix::EventManager::new();
    /// let list = manager.list_event_kinds().unwrap();
    /// ```
    pub fn list_event_kinds(&self) -> Result<Vec<String>> {
        let listeners = self
            .listeners
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event manager: {err}")))?;

        Ok(listeners.keys().cloned().collect::<Vec<_>>())
    }

    /// Adds an event listener.
    ///
    /// # Arguments
    /// - `event_kind`: A string that identifies the type of event this listener is for.
    /// - `listener`: A function or closure that will be called when the event is emitted.
    ///
    /// # Returns
    /// - `Ok(())` if the listener was successfully added.
    /// - `Err(anyhow::Error)` if access to the `Mutex` failed.
    ///
    /// # Example
    /// ```rust
    /// let manager = emitix::EventManager::new();
    /// manager
    ///     .add_listener("Events You Like", |event: String| {
    ///         println!("Event received: {}", event);
    ///     })
    ///     .unwrap();
    /// ```
    pub fn add_listener<F: FnMut(T) + Send + Sync + 'static>(&self, event_kind: &str, listener: F) -> Result<()> {
        let mut listeners = self
            .listeners
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event manager: {err}")))?;

        listeners
            .entry(event_kind.to_string())
            .or_default()
            .push(Box::new(listener));

        Ok(())
    }

    /// Removes a listener for a specific event kind.
    ///
    /// # Arguments
    /// - `event_kind`: A string that identifies the type of event this listener is for.
    /// - `listener`: The listener function or closure to be removed.
    ///
    /// # Returns
    /// - `Ok(())` if the listener was successfully removed.
    /// - `Err(anyhow::Error)` if access to the `Mutex` failed.
    ///
    /// # Example
    /// ```rust
    /// let manager = emitix::EventManager::new();
    /// let listener = |event: String| {
    ///     println!("Event received: {}", event);
    /// };
    ///
    /// manager.add_listener("Events You Like", listener).unwrap();
    /// manager
    ///     .remove_listener("Events You Like", listener)
    ///     .unwrap();
    /// ```
    pub fn remove_listener<F: FnMut(T) + Send + Sync + 'static>(&self, event_kind: &str, listener: F) -> Result<()> {
        let mut listeners = self
            .listeners
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event manager: {err}")))?;

        if let Some(event_listeners) = listeners.get_mut(event_kind) {
            event_listeners.retain(|l| !std::ptr::eq(l.as_ref(), &listener));
        }

        Ok(())
    }

    /// Checks if there are any listeners for a specific event kind.
    /// # Arguments
    /// - `event_kind`: A string that identifies the type of event to check for listeners.
    ///
    /// # Returns
    /// - `Ok(bool)` indicating whether there are listeners for the specified event kind.
    /// - `Err(anyhow::Error)` if access to the `Mutex` failed.
    ///
    /// # Example
    /// ```rust
    /// let manager = emitix::EventManager::new();
    /// let has_listeners = manager.has_listeners("Events You Like").unwrap();
    pub fn has_listeners(&self, event_kind: &str) -> Result<bool> {
        let listeners = self
            .listeners
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event manager: {err}")))?;

        Ok(listeners.contains_key(event_kind))
    }

    /// Returns the number of listeners for a specific event kind.
    ///
    /// # Arguments
    /// - `event_kind`: A string that identifies the type of event whose listeners count is requested.
    ///
    /// # Returns
    /// - `Ok(usize)` representing the number of listeners for the specified event kind.
    /// - `Err(anyhow::Error)` if access to the `Mutex` failed.
    ///
    /// # Example
    /// ```rust
    /// let manager = emitix::EventManager::new();
    /// let count = manager.listeners_count("Events You Like").unwrap();
    pub fn listeners_count(&self, event_kind: &str) -> Result<usize> {
        let listeners = self
            .listeners
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event manager: {err}")))?;

        Ok(listeners.get(event_kind).map_or(0, |l| l.len()))
    }

    /// Clears all listeners for a specific event kind.
    ///
    /// # Arguments
    /// - `event_kind`: A string that identifies the type of event whose listeners should be cleared.
    ///
    /// # Returns
    /// - `Ok(())` if the listeners were successfully cleared.
    /// - `Err(anyhow::Error)` if access to the `Mutex` failed.
    ///
    /// # Example
    /// ```rust
    /// let manager = emitix::EventManager::new();
    /// manager.clear_listeners("Events You Like").unwrap()
    /// ```
    pub fn clear_listeners(&self, event_kind: &str) -> Result<()> {
        let mut listeners = self
            .listeners
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event manager: {err}")))?;

        listeners.remove(event_kind);
        Ok(())
    }

    /// Emits an event to all registered listeners.
    ///
    /// # Arguments
    /// - `event_kind`: A string that identifies the type of event being emitted.
    /// - `event_arg`: The event argument that will be passed to each listener.
    ///
    /// # Returns
    /// - `Ok(())` if the event was successfully emitted.
    /// - `Err(anyhow::Error)` if access to the `Mutex` failed.
    ///
    /// # Example
    /// ```rust
    /// let manager = emitix::EventManager::new();
    /// manager
    ///     .emit("Events You Like", String::from("Test Event"))
    ///     .unwrap();
    /// ```
    pub fn emit(&self, event_kind: &str, event_arg: T) -> Result<()> {
        let mut listeners = self
            .listeners
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event manager: {err}")))?;

        if let Some(event_listeners) = listeners.get_mut(event_kind) {
            for listener in event_listeners.iter_mut() {
                listener(event_arg.clone());
            }
        }

        Ok(())
    }
}
