use std::fmt::Debug;

use dnp3::app::{Listener, MaybeAsync};

pub struct LogListener {
    name: String,
}

impl LogListener {
    pub fn create<T: Debug + 'static>(name: impl Into<String>) -> Box<dyn Listener<T>> {
        Box::new(Self { name: name.into() })
    }
}

impl<T: Debug> Listener<T> for LogListener {
    fn update(&mut self, value: T) -> MaybeAsync<()> {
        tracing::info!("[{}] channel state change: {:?}", self.name, value);
        MaybeAsync::ready(())
    }
}