use super::listener::Listener;
use crate::EventEmitter;
use anyhow::Result;

pub(super) struct EventHubEmitter<T: Clone + Send + Sync + 'static> {
    listener: Listener<T>,
}

impl<T: Clone + Send + Sync + 'static> EventHubEmitter<T> {
    pub(super) fn new(listener: Listener<T>) -> Self {
        Self {
            listener
        }
    }
}

impl<T: Clone + Send + Sync + 'static> EventEmitter<T> for EventHubEmitter<T> {
    fn emit(&self, event_arg: T) -> Result<()> {
        self.listener.call(event_arg)
    }

    fn clone(&self) -> Box<dyn EventEmitter<T>> {
        Box::new(Self {
            listener: self.listener.clone(),
        })
    }
}
