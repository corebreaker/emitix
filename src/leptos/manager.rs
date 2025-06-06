use super::{emitter::LeptosChannelEmitter, registry::ListenerRegistry};
use crate::{EventEmitter, EventManager};
use anyhow::{Result, Error};
use uuid::Uuid;
use log::error;
use leptos::callback::{Callback, Callable};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct LeptosEventChannels<T: Clone + Send + Sync + 'static = ()> {
    registry: Arc<RwLock<ListenerRegistry<T>>>,
}

impl<T: Clone + Send + Sync + 'static> LeptosEventChannels<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: Clone + Send + Sync + 'static> Default for LeptosEventChannels<T> {
    fn default() -> Self {
        Self {
            registry: Arc::new(RwLock::new(ListenerRegistry::new())),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> EventManager<T> for LeptosEventChannels<T> {
    fn list_event_kinds(&self) -> Result<Vec<String>> {
        let registry = self
            .registry
            .read()
            .map_err(|err| Error::msg(format!("Mutex lock failed in Leptos event channels: {err}")))?;

        Ok(registry.listeners().keys().cloned().collect::<Vec<_>>())
    }

    fn has_listeners(&self, event_kind: &str) -> Result<bool> {
        let registry = self
            .registry
            .read()
            .map_err(|err| Error::msg(format!("Mutex lock failed in Leptos event channels: {err}")))?;

        Ok(registry.listeners().contains_key(event_kind))
    }

    fn listeners_count(&self, event_kind: &str) -> Result<usize> {
        let registry = self
            .registry
            .read()
            .map_err(|err| Error::msg(format!("Mutex lock failed in Leptos event channels: {err}")))?;

        Ok(registry.listeners().get(event_kind).map_or(0, |l| l.len()))
    }

    fn clear_listeners(&self) -> Result<()> {
        let mut registry = self
            .registry
            .write()
            .map_err(|err| Error::msg(format!("Mutex lock failed in Leptos event channels: {err}")))?;

        registry.clear();
        Ok(())
    }

    fn add_listener<F: FnMut(T) + Send + Sync + 'static>(&self, event_kind: &str, listener: F) -> Result<Uuid> {
        let mut registry = self
            .registry
            .write()
            .map_err(|err| Error::msg(format!("Mutex lock failed in Leptos event channels: {err}")))?;

        Ok(registry.register_listener(event_kind, listener))
    }

    fn remove_listener(&self, listener_id: Uuid) -> Result<bool> {
        let mut registry = self
            .registry
            .write()
            .map_err(|err| Error::msg(format!("Mutex lock failed in Leptos event channels: {err}")))?;

        Ok(registry.remove_listener(listener_id))
    }

    fn remove_listeners_by_kind(&self, event_kind: &str) -> Result<usize> {
        let mut registry = self
            .registry
            .write()
            .map_err(|err| Error::msg(format!("Mutex lock failed in Leptos event channels: {err}")))?;

        Ok(registry.remove_listeners_by_kind(event_kind))
    }

    fn new_emitter(&self, event_kind: &str) -> Box<dyn EventEmitter<T>> {
        let event_kind = event_kind.to_string();
        let registry = Arc::clone(&self.registry);

        let callback = Callback::new(move |event_arg: T| {
            let event_listeners = {
                let registry = match registry.read() {
                    Ok(lock) => lock,
                    Err(err) => {
                        error!("Failed to lock the registry in Leptos event channels: {err}.");
                        return;
                    }
                };

                registry
                    .listeners()
                    .get(&event_kind)
                    .map(|listeners| listeners.values().cloned().collect::<Vec<_>>())
            };

            if let Some(callbacks) = event_listeners {
                for callback in callbacks {
                    callback.run(event_arg.clone());
                }
            }
        });

        Box::new(LeptosChannelEmitter::new(callback))
    }

    fn new_broadcast_emitter(&self, event_kinds: &[&str]) -> Box<dyn EventEmitter<T>> {
        let event_kinds = if event_kinds.is_empty() {
            None
        } else {
            Some(event_kinds.iter().map(|&s| s.to_string()).collect::<Vec<_>>())
        };

        let registry = Arc::clone(&self.registry);

        let callback = Callback::new(move |event_arg: T| {
            let callbacks = {
                let registry = match registry.read() {
                    Ok(lock) => lock,
                    Err(err) => {
                        error!("Failed to lock the registry in Leptos event channels: {err}.");
                        return;
                    }
                };

                let listeners = registry.listeners();
                let event_kinds = match &event_kinds {
                    Some(list) => list.clone(),
                    None => listeners.keys().cloned().collect::<Vec<_>>(),
                };

                let mut event_listeners: Vec<Callback<T>> = vec![];
                for event_kind in event_kinds {
                    if let Some(callbacks) = listeners.get(&event_kind) {
                        event_listeners.extend(callbacks.values().cloned());
                    }
                }

                event_listeners
            };

            for callback in callbacks {
                callback.run(event_arg.clone());
            }
        });

        Box::new(LeptosChannelEmitter::new(callback))
    }
}
