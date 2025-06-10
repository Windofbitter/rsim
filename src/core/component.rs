use super::types::ComponentId;
use super::event::Event;

pub trait BaseComponent {
    fn component_id(&self) -> &ComponentId;
    fn subscriptions(&self) -> &[&'static str];
    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)>;
}

