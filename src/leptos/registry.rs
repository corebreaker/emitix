use uuid::Uuid;
use leptos::callback::Callback;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub(super) type ListenerMap<T> = HashMap<String, HashMap<Uuid, Callback<T>>>;

pub(super) struct ListenerRegistry<T: Clone + Send + Sync + 'static> {
    listeners: ListenerMap<T>,
    links:     HashMap<Uuid, String>,
}

impl<T: Clone + Send + Sync + 'static> ListenerRegistry<T> {
    pub(super) fn new() -> Self {
        Self {
            listeners: HashMap::new(),
            links:     HashMap::new(),
        }
    }

    pub(super) fn clear(&mut self) {
        self.listeners.clear();
        self.links.clear();
    }

    pub(super) fn listeners(&self) -> &ListenerMap<T> {
        &self.listeners
    }

    pub(super) fn remove_listener(&mut self, listener_id: Uuid) -> bool {
        if let Some(event_kind) = self.links.remove(&listener_id) {
            if let Some(listeners) = self.listeners.get_mut(&event_kind) {
                listeners.remove(&listener_id);
                if listeners.is_empty() {
                    self.listeners.remove(&event_kind);
                }

                return true;
            }
        }

        false
    }

    pub(super) fn remove_listeners_by_kind(&mut self, event_kind: &str) -> usize {
        match self.listeners.remove(event_kind) {
            None => 0,
            Some(listeners) => {
                let sz = listeners.len();
                for listener_id in listeners.keys() {
                    self.links.remove(listener_id);
                }

                sz
            }
        }
    }

    pub(super) fn register_listener<F>(&mut self, event_kind: &str, listener: F) -> Uuid
    where
        F: FnMut(T) + Send + Sync + 'static, {
        let f = Arc::new(RwLock::new(listener));
        let receiver = Callback::new(move |arg| {
            let f = Arc::clone(&f);
            if let Ok(mut caller) = f.write() {
                caller(arg);
            }
        });

        let listener_id = Uuid::new_v4();
        let event_kind = event_kind.to_string();
        let entry = self.listeners.entry(event_kind.clone()).or_default();

        entry.insert(listener_id, receiver);
        self.links.insert(listener_id, event_kind);

        listener_id
    }
}
