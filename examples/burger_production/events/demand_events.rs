use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue, SimulationTime};
use std::collections::HashMap;

pub const GENERATE_ORDER_EVENT: &str = "generate_order";
pub const PLACE_ORDER_EVENT: &str = "place_order";

#[derive(Debug, Clone)]
pub struct GenerateOrderEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub timestamp: SimulationTime,
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

    fn timestamp(&self) -> SimulationTime {
        self.timestamp
    }

    fn data(&self) -> HashMap<String, ComponentValue> {
        HashMap::new()
    }
}

#[derive(Debug, Clone)]
pub struct PlaceOrderEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_id: ComponentId,
    pub timestamp: SimulationTime,
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

    fn timestamp(&self) -> SimulationTime {
        self.timestamp
    }

    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("burger_count".to_string(), ComponentValue::Int(self.burger_count as i64));
        data.insert("order_id".to_string(), ComponentValue::String(self.order_id.clone()));
        data
    }
}