use super::types::ComponentId;
use super::event::{Event, EventType};

pub trait BaseComponent {
    fn component_id(&self) -> &ComponentId;
    fn subscriptions(&self) -> &[EventType];
    fn react_atomic(&mut self, events: Vec<Event>) -> Vec<(Event, u64)>;
}

