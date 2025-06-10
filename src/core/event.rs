use super::types::ComponentId;
use std::fmt::Debug;

pub type EventId = String;

pub trait Event: Debug + Clone {
    fn event_type(&self) -> &'static str;
    fn source_id(&self) -> &ComponentId;
    fn target_ids(&self) -> Option<&[ComponentId]>;
    fn event_id(&self) -> &EventId;
}