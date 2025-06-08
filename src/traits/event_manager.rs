use super::EventEmitter;
use anyhow::Result;
use uuid::Uuid;

pub trait EventManager<T: Clone + Send + Sync + 'static = ()>: Default + Clone + Send + Sync + 'static {
    /// Lists all event kinds that have registered listeners.
    ///
    /// # Returns
    /// - `Ok(Vec<String>)` containing the names of all event kinds with listeners.
    /// - `Err(anyhow::Error)` if access to the underlying data structure fails.
    fn list_event_kinds(&self) -> Result<Vec<String>>;

    /// Checks if there are any listeners for a specific event kind.
    ///
    /// # Arguments
    /// - `event_kind`: A string that identifies the type of event to check for listeners.
    ///
    /// # Returns
    /// - `Ok(bool)` indicating whether there are listeners for the specified event kind.
    /// - `Err(anyhow::Error)` if access to the underlying data structure fails.
    fn has_listeners(&self, event_kind: &str) -> Result<bool>;

    /// Returns the number of listeners for a specific event kind.
    ///
    /// # Arguments
    /// - `event_kind`: A string that identifies the type of event whose listeners count is requested.
    ///
    /// # Returns
    /// - `Ok(usize)` representing the number of listeners for the specified event kind.
    /// - `Err(anyhow::Error)` if access to the underlying data structure fails.
    fn listeners_count(&self, event_kind: &str) -> Result<usize>;

    /// Clears all listeners.
    ///
    /// # Returns
    /// - `Ok(())` if the listeners were successfully cleared.
    /// - `Err(anyhow::Error)` if access to the underlying data structure fails.
    fn clear_listeners(&self) -> Result<()>;

    /// Adds a listener for a specific event kind.
    ///
    /// # Arguments
    /// - `event_kind`: A string that identifies the type of event to listen for.
    /// - `listener`: A function that will be called when the event occurs.
    ///
    /// # Listener Function
    /// The listener function must accept a single argument of type `T`,
    /// which is the event data, and must be `Send`, `Sync`, and `'static`.
    ///
    /// # Returns
    /// - `Ok(Uuid)` which is a unique identifier for the listener.
    /// - `Err(anyhow::Error)` if access to the underlying data structure fails.
    fn add_listener<F: FnMut(T) + Send + Sync + 'static>(&self, event_kind: &str, listener: F) -> Result<Uuid>;

    /// Removes a listener.
    ///
    /// # Arguments
    /// - `listener_id`: A unique identifier for the listener to be removed.
    ///
    /// # Returns
    /// - `Ok(())` if the listener was successfully removed.
    /// - `Err(anyhow::Error)` if access to the underlying data structure fails.
    fn remove_listener(&self, listener_id: Uuid) -> Result<bool>;

    /// Removes all listeners for a specific event kind.
    ///
    /// # Arguments
    /// - `event_kind`: A string that identifies the type of event whose listeners should be removed.
    ///
    /// # Returns
    /// - `Ok(usize)` representing the number of listeners removed.
    /// - `Err(anyhow::Error)` if access to the underlying data structure fails.
    fn remove_listeners_by_kind(&self, event_kind: &str) -> Result<usize>;

    /// Creates a new event emitter for the specified event kind.
    ///
    /// # Arguments
    /// - `event_kind`: A string that identifies the type of event this emitter will handle.
    ///
    /// # Returns
    /// - `Ok(Box<dyn EventEmitter<T>>)` which is a boxed event emitter that can emit events of type `T`.
    /// - `Err(anyhow::Error)` if the emitter could not be created.
    fn new_emitter(&self, event_kind: &str) -> Box<dyn EventEmitter<T>>;

    /// Emits an event of type `T` to all registered listeners for the specified event kinds.
    ///
    /// # Arguments
    /// - `event_kinds`: A slice of strings that identifies the types of events to emit.
    ///
    /// # Returns
    /// - `Ok(Box<dyn EventEmitter<T>>)` which is a boxed event emitter that can emit events of type `T`.
    /// - `Err(anyhow::Error)` if the emitter could not be created.
    fn new_broadcast_emitter(&self, event_kinds: &[&str]) -> Box<dyn EventEmitter<T>>;

    /// Returns a null emitter used as default emitter.
    ///
    /// Null emitters do not emit events and do not have any listeners.
    /// It's like, we call `Self::default().new_emitter("")`.
    ///
    /// This is useful when no listeners are registered for an event kind.
    ///
    /// # Returns
    /// - `Box<dyn EventEmitter<T>>` which is a boxed event emitter that does not emit events.
    fn new_null_emitter() -> Box<dyn EventEmitter<T>>;
}
