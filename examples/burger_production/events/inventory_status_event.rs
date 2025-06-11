use rsim::core::event::{Event, EventId, EventType};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct InventoryStatusEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
    buffer_type: String,
    current_count: i32,
    capacity: i32,
}

impl InventoryStatusEvent {
    pub fn new(
        source_id: ComponentId,
        target_ids: Option<Vec<ComponentId>>,
        buffer_type: String,
        current_count: i32,
        capacity: i32,
    ) -> Self {
        Self {
            id: format!("inventory_status_{}", uuid::Uuid::new_v4()),
            source_id,
            target_ids,
            buffer_type,
            current_count,
            capacity,
        }
    }

    pub fn buffer_type(&self) -> &str {
        &self.buffer_type
    }

    pub fn current_count(&self) -> i32 {
        self.current_count
    }

    pub fn capacity(&self) -> i32 {
        self.capacity
    }
}

impl Event for InventoryStatusEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        "InventoryStatusEvent"
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        self.target_ids.clone()
    }

    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("buffer_type".to_string(), ComponentValue::String(self.buffer_type.clone()));
        data.insert("current_count".to_string(), ComponentValue::Int(self.current_count as i64));
        data.insert("capacity".to_string(), ComponentValue::Int(self.capacity as i64));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}