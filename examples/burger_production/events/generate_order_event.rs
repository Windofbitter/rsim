use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GenerateOrderEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
}

impl GenerateOrderEvent {
    pub fn new(source_id: ComponentId, target_ids: Option<Vec<ComponentId>>) -> Self {
        Self {
            id: format!("generate_order_{}", uuid::Uuid::new_v4()),
            source_id,
            target_ids,
        }
    }
}

impl Event for GenerateOrderEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        "GenerateOrderEvent"
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        self.target_ids.clone()
    }

    fn data(&self) -> HashMap<String, ComponentValue> {
        HashMap::new()
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}
