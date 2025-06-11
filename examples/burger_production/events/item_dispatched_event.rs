use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ItemDispatchedEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
    item_type: String,
    item_id: String,
    success: bool,
}

impl ItemDispatchedEvent {
    pub fn new(
        source_id: ComponentId,
        target_id: ComponentId,
        item_type: String,
        item_id: String,
        success: bool,
    ) -> Self {
        Self {
            id: format!("item_dispatched_{}", uuid::Uuid::new_v4()),
            source_id,
            target_ids: Some(vec![target_id]),
            item_type,
            item_id,
            success,
        }
    }

    pub fn item_type(&self) -> &str {
        &self.item_type
    }

    pub fn item_id(&self) -> &str {
        &self.item_id
    }

    pub fn success(&self) -> bool {
        self.success
    }
}

impl Event for ItemDispatchedEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        "ItemDispatchedEvent"
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
        data.insert("success".to_string(), ComponentValue::Bool(self.success));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
} 