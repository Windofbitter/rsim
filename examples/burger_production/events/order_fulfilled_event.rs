use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OrderFulfilledEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
    order_id: String,
}

impl OrderFulfilledEvent {
    pub fn new(
        source_id: ComponentId,
        target_ids: Option<Vec<ComponentId>>,
        order_id: String,
    ) -> Self {
        Self {
            id: format!("order_fulfilled_{}", uuid::Uuid::new_v4()),
            source_id,
            target_ids,
            order_id,
        }
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
        data.insert(
            "order_id".to_string(),
            ComponentValue::String(self.order_id.clone()),
        );
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}
