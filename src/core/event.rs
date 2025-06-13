use super::types::{ComponentId, ComponentValue};
use std::collections::HashMap;
use std::fmt::Debug;

pub type EventId = String;

pub type EventType = String;

pub trait Event: Debug {
    fn id(&self) -> &EventId;
    fn event_type(&self) -> &str;
    fn source_id(&self) -> &ComponentId;
    fn target_ids(&self) -> Option<Vec<ComponentId>>;
    fn data(&self) -> HashMap<String, ComponentValue>;
    fn clone_event(&self) -> Box<dyn Event>;
}

/// Core simulation lifecycle event for cycle advancement
#[derive(Debug, Clone)]
pub struct CycleAdvancedEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
    old_cycle: u64,
    new_cycle: u64,
}

impl CycleAdvancedEvent {
    pub fn new(source_id: ComponentId, target_ids: Option<Vec<ComponentId>>, old_cycle: u64, new_cycle: u64) -> Self {
        Self {
            id: format!("cycle_advanced_{}_to_{}", old_cycle, new_cycle),
            source_id,
            target_ids,
            old_cycle,
            new_cycle,
        }
    }
}

impl Event for CycleAdvancedEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        "CycleAdvancedEvent"
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        self.target_ids.clone()
    }

    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("old_cycle".to_string(), ComponentValue::Int(self.old_cycle as i64));
        data.insert("new_cycle".to_string(), ComponentValue::Int(self.new_cycle as i64));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}
