use rsim::core::event::{Event, EventId, EventType};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProductionRequestEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
    item_type: String,
    quantity: i32,
}

impl ProductionRequestEvent {
    pub fn new(
        source_id: ComponentId,
        target_ids: Option<Vec<ComponentId>>,
        item_type: String,
        quantity: i32,
    ) -> Self {
        Self {
            id: format!("production_request_{}", uuid::Uuid::new_v4()),
            source_id,
            target_ids,
            item_type,
            quantity,
        }
    }

    pub fn item_type(&self) -> &str {
        &self.item_type
    }

    pub fn quantity(&self) -> i32 {
        self.quantity
    }
}

impl Event for ProductionRequestEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        "ProductionRequestEvent"
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
        data.insert("quantity".to_string(), ComponentValue::Int(self.quantity as i64));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}