use anyhow::{Error, Result};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(super) struct Listener<T: Clone + Send + Sync + 'static> {
    callback: Arc<Mutex<Box<dyn FnMut(T) -> Result<()> + Send + Sync>>>,
}

impl<T: Clone + Send + Sync + 'static> Listener<T> {
    pub(super) fn new<F: FnMut(T) -> Result<()> + Send + Sync + 'static>(callback: F) -> Self {
        Self {
            callback: Arc::new(Mutex::new(Box::new(callback))),
        }
    }

    pub(super) fn call(&self, event_arg: T) -> Result<()> {
        match self.callback.lock() {
            Err(e) => Err(Error::msg(format!("Failed to lock listener callback: {e}"))),
            Ok(mut cb) => cb(event_arg),
        }
    }
}
