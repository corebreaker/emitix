use super::registry::ListenerRegistry;
use crate::EventEmitter;
use anyhow::{Error, Result};
use std::sync::{Arc, Mutex};

pub(super) struct EventHubBroadcaster<T: Clone + Send + Sync + 'static> {
    registry:    Arc<Mutex<ListenerRegistry<T>>>,
    event_kinds: Vec<String>,
}

impl<T: Clone + Send + Sync + 'static> EventHubBroadcaster<T> {
    pub(super) fn new(registry: Arc<Mutex<ListenerRegistry<T>>>, event_kinds: Vec<String>) -> Self {
        Self {
            registry,
            event_kinds,
        }
    }
}

impl<T: Clone + Send + Sync + 'static> EventEmitter<T> for EventHubBroadcaster<T> {
    fn emit(&self, event_arg: T) -> Result<()> {
        let mut registry = self
            .registry
            .lock()
            .map_err(|err| Error::msg(format!("Mutex lock failed in event hub: {err}")))?;

        let listeners = registry.listeners_mut();
        let event_kinds = if self.event_kinds.is_empty() {
            listeners.keys().cloned().collect::<Vec<_>>()
        } else {
            self.event_kinds.clone()
        };

        for event_kind in event_kinds {
            if let Some(event_listeners) = listeners.get_mut(&event_kind) {
                for listener in event_listeners.values_mut() {
                    listener(event_arg.clone());
                }
            }
        }

        Ok(())
    }
}
