use rsim::core::event::{Event, EventId, EventType};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ItemDroppedEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
    item_type: String,
    item_id: String,
    reason: String,
}

impl ItemDroppedEvent {
    pub fn new(
        source_id: ComponentId,
        target_ids: Option<Vec<ComponentId>>,
        item_type: String,
        item_id: String,
        reason: String,
    ) -> Self {
        Self {
            id: format!("item_dropped_{}", uuid::Uuid::new_v4()),
            source_id,
            target_ids,
            item_type,
            item_id,
            reason,
        }
    }

    pub fn item_type(&self) -> &str {
        &self.item_type
    }

    pub fn item_id(&self) -> &str {
        &self.item_id
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl Event for ItemDroppedEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        "ItemDroppedEvent"
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        self.target_ids.clone()
    }

    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("item_type".to_string(), ComponentValue::String(self.item_type.clone()));
        data.insert("item_id".to_string(), ComponentValue::String(self.item_id.clone()));
        data.insert("reason".to_string(), ComponentValue::String(self.reason.clone()));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}