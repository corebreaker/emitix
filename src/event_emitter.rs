use anyhow::Result;

pub trait EventEmitter<T: Clone + Send + Sync + 'static = ()>: Send + Sync + 'static {
    /// Emits an event of type `T` to all registered listeners for the specified event kind.
    ///
    /// # Arguments
    /// - `event_arg`: The event data of type `T` to be emitted.
    ///
    /// # Returns
    /// - `Ok(())` if the event was successfully emitted to all listeners.
    /// - `Err(anyhow::Error)` if access to the underlying data structure fails,
    ///     or if no listeners are found for the event kind.
    fn emit(&self, event_arg: T) -> Result<()>;
}
