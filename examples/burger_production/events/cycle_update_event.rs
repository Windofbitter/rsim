use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CycleUpdateEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
    cycle: u64,
}

impl CycleUpdateEvent {
    pub fn new(source_id: ComponentId, target_ids: Option<Vec<ComponentId>>, cycle: u64) -> Self {
        Self {
            id: format!("cycle_update_{}", uuid::Uuid::new_v4()),
            source_id,
            target_ids,
            cycle,
        }
    }
}

impl Event for CycleUpdateEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        "CycleUpdateEvent"
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        self.target_ids.clone()
    }

    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("cycle".to_string(), ComponentValue::Int(self.cycle as i64));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}