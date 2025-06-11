use rsim::core::event::{Event, EventId, EventType};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PlaceOrderEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
    order_id: String,
    quantity: i32,
}

impl PlaceOrderEvent {
    pub fn new(
        source_id: ComponentId,
        target_ids: Option<Vec<ComponentId>>,
        order_id: String,
        quantity: i32,
    ) -> Self {
        Self {
            id: format!("place_order_{}", uuid::Uuid::new_v4()),
            source_id,
            target_ids,
            order_id,
            quantity,
        }
    }

    pub fn order_id(&self) -> &str {
        &self.order_id
    }

    pub fn quantity(&self) -> i32 {
        self.quantity
    }
}

impl Event for PlaceOrderEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        "PlaceOrderEvent"
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        self.target_ids.clone()
    }

    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("order_id".to_string(), ComponentValue::String(self.order_id.clone()));
        data.insert("quantity".to_string(), ComponentValue::Int(self.quantity as i64));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}