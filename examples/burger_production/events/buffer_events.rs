use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

pub const ITEM_ADDED_EVENT: &str = "item_added";
pub const BUFFER_FULL_EVENT: &str = "buffer_full";
pub const BUFFER_SPACE_AVAILABLE_EVENT: &str = "buffer_space_available";
pub const REQUEST_ITEM_EVENT: &str = "request_item";

#[derive(Debug, Clone)]
pub struct ItemAddedEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub current_count: u32,
    pub item_type: String,
    pub item_id: String,
}

impl Event for ItemAddedEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        ITEM_ADDED_EVENT
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        None
    }


    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("current_count".to_string(), ComponentValue::Int(self.current_count as i64));
        data.insert("item_type".to_string(), ComponentValue::String(self.item_type.clone()));
        data.insert("item_id".to_string(), ComponentValue::String(self.item_id.clone()));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct BufferFullEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_id: ComponentId,
    pub capacity: u32,
}

impl Event for BufferFullEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        BUFFER_FULL_EVENT
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        Some(vec![self.target_id.clone()])
    }


    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("capacity".to_string(), ComponentValue::Int(self.capacity as i64));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct BufferSpaceAvailableEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_id: ComponentId,
    pub available_space: u32,
}

impl Event for BufferSpaceAvailableEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        BUFFER_SPACE_AVAILABLE_EVENT
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        Some(vec![self.target_id.clone()])
    }


    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("available_space".to_string(), ComponentValue::Int(self.available_space as i64));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct RequestItemEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_id: ComponentId,
    pub quantity: u32,
}

impl Event for RequestItemEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        REQUEST_ITEM_EVENT
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        Some(vec![self.target_id.clone()])
    }


    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("quantity".to_string(), ComponentValue::Int(self.quantity as i64));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}