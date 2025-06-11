use super::types::{ComponentId, ComponentValue};
use std::collections::HashMap;
use std::fmt::Debug;

pub type EventId = String;

pub type EventType = String;

pub trait Event: Debug {
    fn id(&self) -> &EventId;
    fn event_type(&self) -> &str;
    fn source_id(&self) -> &ComponentId;
    fn target_ids(&self) -> Option<Vec<ComponentId>>;
    fn data(&self) -> HashMap<String, ComponentValue>;
    fn clone_event(&self) -> Box<dyn Event>;
}
