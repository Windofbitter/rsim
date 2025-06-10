use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue, SimulationTime};
use std::collections::HashMap;

pub const START_ASSEMBLY_EVENT: &str = "start_assembly";
pub const BURGER_READY_EVENT: &str = "burger_ready";

#[derive(Debug, Clone)]
pub struct StartAssemblyEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub timestamp: SimulationTime,
    pub meat_id: String,
    pub bread_id: String,
}

impl Event for StartAssemblyEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        START_ASSEMBLY_EVENT
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
        let mut data = HashMap::new();
        data.insert("meat_id".to_string(), ComponentValue::String(self.meat_id.clone()));
        data.insert("bread_id".to_string(), ComponentValue::String(self.bread_id.clone()));
        data
    }
}

#[derive(Debug, Clone)]
pub struct BurgerReadyEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_id: ComponentId,
    pub timestamp: SimulationTime,
    pub burger_id: String,
}

impl Event for BurgerReadyEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        BURGER_READY_EVENT
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
        data.insert("burger_id".to_string(), ComponentValue::String(self.burger_id.clone()));
        data.insert("item_type".to_string(), ComponentValue::String("burger".to_string()));
        data
    }
}