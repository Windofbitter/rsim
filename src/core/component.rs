use super::event::Event;
use super::types::ComponentId;

pub trait BaseComponent {
    fn component_id(&self) -> &ComponentId;
    fn subscriptions(&self) -> &[&'static str];
    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)>;
}
