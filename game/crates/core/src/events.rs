//! Event queue drained by the host every tick. Excluded from the state hash.

use serde::{Deserialize, Serialize};

/// A simulation event for the host. WP-004 and later add variants.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {}

/// Events emitted since the host last drained the queue.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EventQueue {
    events: Vec<Event>,
}

impl EventQueue {
    pub fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Removes and returns all queued events in emission order.
    pub fn drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
