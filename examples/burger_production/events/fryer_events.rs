use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue, SimulationTime};
use std::collections::HashMap;

pub const START_FRYING_EVENT: &str = "start_frying";
pub const MEAT_READY_EVENT: &str = "meat_ready";

#[derive(Debug, Clone)]
pub struct StartFryingEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub timestamp: SimulationTime,
}

impl Event for StartFryingEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        START_FRYING_EVENT
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
pub struct MeatReadyEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_id: ComponentId,
    pub timestamp: SimulationTime,
    pub meat_id: String,
}

impl Event for MeatReadyEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        MEAT_READY_EVENT
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        Some(vec![self.target_id.clone()])
    }


    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("meat_id".to_string(), ComponentValue::String(self.meat_id.clone()));
        data.insert("item_type".to_string(), ComponentValue::String("meat".to_string()));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}