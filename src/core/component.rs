use super::event::Event;
use super::types::ComponentId;

pub trait BaseComponent {
    fn component_id(&self) -> &ComponentId;
    fn subscriptions(&self) -> &[&'static str];
    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)>;
    
    /// Returns a list of event types that this component may emit.
    /// This is used to build the dependency graph for parallel execution.
    fn emitted_events(&self) -> &[&'static str] {
        &[] // Default implementation returns an empty list
    }
}
