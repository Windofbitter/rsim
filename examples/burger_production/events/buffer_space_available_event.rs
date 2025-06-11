use rsim::core::event::{Event, EventId, EventType};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BufferSpaceAvailableEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
    buffer_type: String,
}

impl BufferSpaceAvailableEvent {
    pub fn new(
        source_id: ComponentId,
        target_ids: Option<Vec<ComponentId>>,
        buffer_type: String,
    ) -> Self {
        Self {
            id: format!("buffer_space_available_{}", uuid::Uuid::new_v4()),
            source_id,
            target_ids,
            buffer_type,
        }
    }

    pub fn buffer_type(&self) -> &str {
        &self.buffer_type
    }
}

impl Event for BufferSpaceAvailableEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        "BufferSpaceAvailableEvent"
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        self.target_ids.clone()
    }

    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("buffer_type".to_string(), ComponentValue::String(self.buffer_type.clone()));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}