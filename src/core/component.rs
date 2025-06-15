use super::event::Event;
use super::types::ComponentId;

pub trait BaseComponent: Send {
    fn component_id(&self) -> &ComponentId;
    fn subscriptions(&self) -> &[&'static str];
    fn emitted_events(&self) -> &[&'static str];
    fn react_atomic(&mut self, events: Vec<Box<dyn Event + Send + Sync>>) -> Vec<(Box<dyn Event + Send + Sync>, u64)>;
}
