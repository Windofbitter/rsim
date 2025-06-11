use rsim::core::event::{Event, EventId, EventType};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OrderFulfilledEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
    order_id: String,
    fulfillment_time: u64,
}

impl OrderFulfilledEvent {
    pub fn new(
        source_id: ComponentId,
        target_ids: Option<Vec<ComponentId>>,
        order_id: String,
        fulfillment_time: u64,
    ) -> Self {
        Self {
            id: format!("order_fulfilled_{}", uuid::Uuid::new_v4()),
            source_id,
            target_ids,
            order_id,
            fulfillment_time,
        }
    }

    pub fn order_id(&self) -> &str {
        &self.order_id
    }

    pub fn fulfillment_time(&self) -> u64 {
        self.fulfillment_time
    }
}

impl Event for OrderFulfilledEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        "OrderFulfilledEvent"
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
        data.insert("fulfillment_time".to_string(), ComponentValue::Int(self.fulfillment_time as i64));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}