use super::registry::ListenerRegistry;
use crate::EventEmitter;
use anyhow::{Error, Result};
use std::sync::{Arc, RwLock};

pub(super) struct EventHubEmitter<T: Clone + Send + Sync + 'static> {
    registry:   Arc<RwLock<ListenerRegistry<T>>>,
    event_kind: String,
}

impl<T: Clone + Send + Sync + 'static> EventHubEmitter<T> {
    pub(super) fn new(registry: Arc<RwLock<ListenerRegistry<T>>>, event_kind: String) -> Self {
        Self {
            registry,
            event_kind,
        }
    }
}

impl<T: Clone + Send + Sync + 'static> EventEmitter<T> for EventHubEmitter<T> {
    fn emit(&self, event_arg: T) -> Result<()> {
        let mut registry = self
            .registry
            .write()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event hub: {err}")))?;

        if let Some(event_listeners) = registry.listeners_mut().get_mut(&self.event_kind) {
            for listener in event_listeners.values_mut() {
                listener(event_arg.clone());
            }
        }

        Ok(())
    }
}
