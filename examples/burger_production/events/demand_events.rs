use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

pub const GENERATE_ORDER_EVENT: &str = "generate_order";
pub const PLACE_ORDER_EVENT: &str = "place_order";
pub const PRODUCTION_REQUEST_EVENT: &str = "production_request";
pub const ORDER_COMPLETED_EVENT: &str = "order_completed";

#[derive(Debug, Clone)]
pub struct GenerateOrderEvent {
    pub id: EventId,
    pub source_id: ComponentId,
}

impl Event for GenerateOrderEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        GENERATE_ORDER_EVENT
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        None
    }


    fn data(&self) -> HashMap<String, ComponentValue> {
        HashMap::new()
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct PlaceOrderEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_id: ComponentId,
    pub burger_count: u32,
    pub order_id: String,
}

impl Event for PlaceOrderEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        PLACE_ORDER_EVENT
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        Some(vec![self.target_id.clone()])
    }


    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("burger_count".to_string(), ComponentValue::Int(self.burger_count as i64));
        data.insert("order_id".to_string(), ComponentValue::String(self.order_id.clone()));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct ProductionRequestEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_id: ComponentId,
    pub item_type: String,
    pub quantity: u32,
    pub order_id: String,
}

impl Event for ProductionRequestEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        PRODUCTION_REQUEST_EVENT
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        Some(vec![self.target_id.clone()])
    }

    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("item_type".to_string(), ComponentValue::String(self.item_type.clone()));
        data.insert("quantity".to_string(), ComponentValue::Int(self.quantity as i64));
        data.insert("order_id".to_string(), ComponentValue::String(self.order_id.clone()));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct OrderCompletedEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_id: ComponentId,
    pub order_id: String,
    pub burger_count: u32,
    pub completion_time: u64,
}

impl Event for OrderCompletedEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        ORDER_COMPLETED_EVENT
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        Some(vec![self.target_id.clone()])
    }

    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("order_id".to_string(), ComponentValue::String(self.order_id.clone()));
        data.insert("burger_count".to_string(), ComponentValue::Int(self.burger_count as i64));
        data.insert("completion_time".to_string(), ComponentValue::Int(self.completion_time as i64));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}