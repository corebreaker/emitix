use super::{emitter::EventHubEmitter, broadcaster::EventHubBroadcaster, registry::ListenerRegistry};
use crate::{EventEmitter, EventManager};
use anyhow::{Result, Error};
use uuid::Uuid;
use std::sync::{Arc, Mutex};

/// `EventHub` is a thread-safe structure for managing events.
///
/// # Features
/// - Allows adding event listeners.
/// - Emits events to all registered listeners.
/// - Uses `Arc` and `Mutex` to ensure thread safety.
///
/// # Example Usage
/// ```rust
/// use emitix::{event_hub::EventHub, EventManager};
///
/// let manager = EventHub::default();
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
pub struct EventHub<T: Clone + Send + Sync + 'static = ()> {
    registry: Arc<Mutex<ListenerRegistry<T>>>,
}

impl<T: Clone + Send + Sync + 'static> EventHub<T> {
    /// Creates a new instance of `EventHub`.
    ///
    /// # Returns
    /// A new, empty instance of `EventHub` ready to register listeners.
    pub fn new() -> Self {
        Self::default()
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
    /// use emitix::event_hub::EventHub;
    ///
    /// let manager = EventHub::default();
    /// manager
    ///     .emit("Events You Like", String::from("Test Event"))
    ///     .unwrap();
    /// ```
    pub fn emit(&self, event_kind: &str, event_arg: T) -> Result<()> {
        let mut registry = self
            .registry
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event hub: {err}")))?;

        if let Some(event_listeners) = registry.listeners_mut().get_mut(event_kind) {
            for listener in event_listeners.values_mut() {
                listener(event_arg.clone());
            }
        }

        Ok(())
    }
}

impl<T: Clone + Send + Sync + 'static> Default for EventHub<T> {
    fn default() -> Self {
        Self {
            registry: Arc::new(Mutex::new(ListenerRegistry::new())),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> EventManager<T> for EventHub<T> {
    /// Lists all event kinds that have registered listeners.
    ///
    /// # Returns
    /// - `Ok(Vec<String>)` containing the names of all event kinds.
    /// - `Err(anyhow::Error)` if access to the `Mutex` failed.
    ///
    /// # Example
    /// ```rust
    /// use emitix::{event_hub::EventHub, EventManager};
    ///
    /// let manager = EventHub::default();
    /// let list = manager.list_event_kinds().unwrap();
    /// ```
    fn list_event_kinds(&self) -> Result<Vec<String>> {
        let registry = self
            .registry
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event hub: {err}")))?;

        Ok(registry.listeners().keys().cloned().collect::<Vec<_>>())
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
    /// use emitix::{event_hub::EventHub, EventManager};
    ///
    /// let manager = EventHub::default();
    /// let has_listeners = manager.has_listeners("Events You Like").unwrap();
    fn has_listeners(&self, event_kind: &str) -> Result<bool> {
        let registry = self
            .registry
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event hub: {err}")))?;

        Ok(registry.listeners().contains_key(event_kind))
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
    /// use emitix::{event_hub::EventHub, EventManager};
    ///
    /// let manager = EventHub::default();
    /// let count = manager.listeners_count("Events You Like").unwrap();
    fn listeners_count(&self, event_kind: &str) -> Result<usize> {
        let registry = self
            .registry
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event hub: {err}")))?;

        Ok(registry.listeners().get(event_kind).map_or(0, |l| l.len()))
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
    /// use emitix::{event_hub::EventHub, EventManager};
    ///
    /// let manager = EventHub::default();
    /// manager.clear_listeners().unwrap()
    /// ```
    fn clear_listeners(&self) -> Result<()> {
        let mut registry = self
            .registry
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event hub: {err}")))?;

        registry.clear();
        Ok(())
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
    /// use emitix::{event_hub::EventHub, EventManager};
    ///
    /// let manager = EventHub::default();
    /// manager
    ///     .add_listener("Events You Like", |event: String| {
    ///         println!("Event received: {}", event);
    ///     })
    ///     .unwrap();
    /// ```
    fn add_listener<F: FnMut(T) + Send + Sync + 'static>(&self, event_kind: &str, listener: F) -> Result<Uuid> {
        let mut registry = self
            .registry
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event hub: {err}")))?;

        Ok(registry.register_listener(event_kind, listener))
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
    /// use emitix::{event_hub::EventHub, EventManager};
    ///
    /// let manager = EventHub::default();
    /// let listener = |event: String| {
    ///     println!("Event received: {}", event);
    /// };
    ///
    /// let listener_id = manager.add_listener("Events You Like", listener).unwrap();
    /// manager.remove_listener(listener_id).unwrap();
    /// ```
    fn remove_listener(&self, listener_id: Uuid) -> Result<bool> {
        let mut registry = self
            .registry
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event hub: {err}")))?;

        Ok(registry.remove_listener(listener_id))
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
    /// use emitix::{event_hub::EventHub, EventManager};
    ///
    /// let manager = EventHub::default();
    /// manager.remove_listeners_by_kind("Events You Like").unwrap()
    /// ```
    fn remove_listeners_by_kind(&self, event_kind: &str) -> Result<usize> {
        let mut registry = self
            .registry
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event hub: {err}")))?;

        Ok(registry.remove_listeners_by_kind(event_kind))
    }

    fn new_emitter(&self, event_kind: &str) -> Box<dyn EventEmitter<T>> {
        Box::new(EventHubEmitter::new(Arc::clone(&self.registry), event_kind.to_string()))
    }

    fn new_broadcast_emitter(&self, event_kinds: &[&str]) -> Box<dyn EventEmitter<T>> {
        let event_kinds = event_kinds.iter().map(|&s| s.to_string()).collect::<Vec<_>>();

        Box::new(EventHubBroadcaster::new(Arc::clone(&self.registry), event_kinds))
    }
}
