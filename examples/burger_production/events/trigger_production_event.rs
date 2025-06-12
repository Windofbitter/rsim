use rsim::core::event::{Event, EventId, EventType};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TriggerProductionEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
}

impl TriggerProductionEvent {
    pub const TYPE: &'static str = "TriggerProductionEvent";
    
    pub fn new(source_id: ComponentId, target_ids: Option<Vec<ComponentId>>) -> Self {
        Self {
            id: format!("trigger_production_{}", uuid::Uuid::new_v4()),
            source_id,
            target_ids,
        }
    }
}

impl Event for TriggerProductionEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        "TriggerProductionEvent"
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