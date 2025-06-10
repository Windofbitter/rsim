use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

pub const START_BAKING_EVENT: &str = "start_baking";
pub const BREAD_READY_EVENT: &str = "bread_ready";

#[derive(Debug, Clone)]
pub struct StartBakingEvent {
    pub id: EventId,
    pub source_id: ComponentId,
}

impl Event for StartBakingEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        START_BAKING_EVENT
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
pub struct BreadReadyEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_id: ComponentId,
    pub bread_id: String,
}

impl Event for BreadReadyEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        BREAD_READY_EVENT
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        Some(vec![self.target_id.clone()])
    }


    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("bread_id".to_string(), ComponentValue::String(self.bread_id.clone()));
        data.insert("item_type".to_string(), ComponentValue::String("bread".to_string()));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}