use crate::EventEmitter;
use anyhow::Result;
use leptos::callback::{Callback, Callable};

pub(super) struct LeptosChannelEmitter<T: Clone + Send + Sync + 'static> {
    callback: Callback<T>,
}

impl<T: Clone + Send + Sync + 'static> LeptosChannelEmitter<T> {
    pub(super) fn new(callback: Callback<T>) -> Self {
        Self {
            callback,
        }
    }
}

impl<T: Clone + Send + Sync + 'static> EventEmitter<T> for LeptosChannelEmitter<T> {
    fn emit(&self, event_arg: T) -> Result<()> {
        self.callback.run(event_arg);

        Ok(())
    }

    fn clone(&self) -> Box<dyn EventEmitter<T>> {
        Box::new(Self {
            callback: self.callback.clone(),
        })
    }
}
