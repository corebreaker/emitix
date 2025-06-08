use super::{emitter::EventHubEmitter, listener::Listener, registry::ListenerRegistry};
use crate::{EventEmitter, EventManager};
use anyhow::{Result, Error};
use uuid::Uuid;
use std::sync::{Arc, RwLock};

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
    registry: Arc<RwLock<ListenerRegistry<T>>>,
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
        let listeners = self
            .registry
            .read()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event hub: {err}")))?
            .listeners()
            .get(event_kind)
            .map(|list| list.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        let mut errors = vec![];
        for listener in listeners {
            if let Err(err) = listener.call(event_arg.clone()) {
                errors.push(err);
            }
        }

        if !errors.is_empty() {
            return Err(Error::msg(format!(
                "Failed to emit event '{event_kind}':\n{}",
                errors.into_iter().map(|err| format!("\n  - {err}")).collect::<String>(),
            )));
        }

        Ok(())
    }
}

impl<T: Clone + Send + Sync + 'static> Default for EventHub<T> {
    fn default() -> Self {
        Self {
            registry: Arc::new(RwLock::new(ListenerRegistry::new())),
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
            .read()
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
            .read()
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
            .read()
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
            .write()
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
            .write()
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
            .write()
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
            .write()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event hub: {err}")))?;

        Ok(registry.remove_listeners_by_kind(event_kind))
    }

    /// Creates a new event emitter for a specific event kind.
    ///
    /// # Arguments
    /// - `event_kind`: A string that identifies the type of event this emitter will handle.
    ///
    /// # Returns
    /// - `Box<dyn EventEmitter<T>>` which is a boxed trait object that implements the `EventEmitter` trait.
    ///
    /// # Example
    /// ```rust
    /// use emitix::{event_hub::EventHub, EventManager};
    ///
    /// let manager = EventHub::default();
    /// let emitter = manager.new_emitter("Events You Like");
    /// emitter.emit(()).unwrap()
    /// ```
    fn new_emitter(&self, event_kind: &str) -> Box<dyn EventEmitter<T>> {
        let event_kind = event_kind.to_string();
        let registry = Arc::clone(&self.registry);
        let listener = Listener::new(move |event_arg: T| -> Result<()> {
            let listeners = registry
                .read()
                .map_err(|err| {
                    let msg = format!("Mutex lock failed in event hub for kind `{event_kind}`: {err}");

                    Error::msg(msg)
                })?
                .listeners()
                .get(&event_kind)
                .map(|listeners| listeners.values().cloned().collect::<Vec<_>>())
                .unwrap_or_default();

            let mut errors = vec![];
            for listener in listeners {
                if let Err(err) = listener.call(event_arg.clone()) {
                    errors.push(err);
                }
            }

            if !errors.is_empty() {
                return Err(Error::msg(format!(
                    "Failed to emit event '{event_kind}':{errors}",
                    errors = errors.into_iter().map(|err| format!("\n  - {err}")).collect::<String>(),
                )));
            }

            Ok(())
        });

        Box::new(EventHubEmitter::new(listener))
    }

    /// Creates a new event broadcaster that emits events to multiple listeners.
    ///
    /// # Arguments
    /// - `event_kinds`: A slice of strings that identifies the types of events this broadcaster will handle.
    ///
    /// # Returns
    /// - `Box<dyn EventEmitter<T>>` which is a boxed trait object that implements the `EventEmitter` trait.
    ///
    /// # Example
    /// ```rust
    /// use emitix::{event_hub::EventHub, EventManager};
    ///
    /// let manager = EventHub::default();
    /// let broadcaster = manager.new_broadcast_emitter(&["Events You Like", "Another Event Kind"]);
    /// broadcaster.emit(()).unwrap()
    /// ```
    fn new_broadcast_emitter(&self, event_kinds: &[&str]) -> Box<dyn EventEmitter<T>> {
        let event_kinds = if event_kinds.is_empty() {
            None
        } else {
            Some(event_kinds.iter().map(|&s| s.to_string()).collect::<Vec<_>>())
        };

        let registry = Arc::clone(&self.registry);
        let listener = Listener::new(move |event_arg: T| -> Result<()> {
            let (listeners, event_kinds) = {
                let registry = registry.read().map_err(|err| {
                    let event_kinds = event_kinds.as_ref().map(|l| l.join(", ")).unwrap_or_default();
                    let msg = format!("Mutex lock failed in event hub for kind `{event_kinds}`: {err}");

                    Error::msg(msg)
                })?;

                let listeners = registry.listeners();
                let kinds_to_process = match &event_kinds {
                    Some(list) => list.clone(),
                    None => listeners.keys().cloned().collect::<Vec<_>>(),
                };

                let mut event_listeners = Vec::new();
                for event_kind in &kinds_to_process {
                    if let Some(callbacks) = listeners.get(event_kind) {
                        event_listeners.extend(callbacks.values().cloned());
                    }
                }

                (event_listeners, kinds_to_process.join(", "))
            };

            let mut errors = vec![];
            for listener in listeners {
                if let Err(err) = listener.call(event_arg.clone()) {
                    errors.push(err);
                }
            }

            if !errors.is_empty() {
                return Err(Error::msg(format!(
                    "Failed to emit event from hub for kinds '{event_kinds}':{errors}",
                    errors = errors.into_iter().map(|err| format!("\n  - {err}")).collect::<String>(),
                )));
            }

            Ok(())
        });

        Box::new(EventHubEmitter::new(listener))
    }

    /// Returns a null emitter used as default emitter.
    ///
    /// # Returns
    /// - `Box<dyn EventEmitter<T>>` which is a boxed trait object that implements the `EventEmitter` trait.
    fn new_null_emitter() -> Box<dyn EventEmitter<T>> {
        Box::new(EventHubEmitter::new(Listener::new(|_| Ok(()))))
    }
}
