use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ItemAddedEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
    buffer_type: String,
    item_id: String,
    current_count: i32,
}

impl ItemAddedEvent {
    pub fn new(
        source_id: ComponentId,
        target_ids: Option<Vec<ComponentId>>,
        buffer_type: String,
        item_id: String,
        current_count: i32,
    ) -> Self {
        Self {
            id: format!("item_added_{}", uuid::Uuid::new_v4()),
            source_id,
            target_ids,
            buffer_type,
            item_id,
            current_count,
        }
    }
}

impl Event for ItemAddedEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        "ItemAddedEvent"
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
            "buffer_type".to_string(),
            ComponentValue::String(self.buffer_type.clone()),
        );
        data.insert(
            "item_id".to_string(),
            ComponentValue::String(self.item_id.clone()),
        );
        data.insert(
            "current_count".to_string(),
            ComponentValue::Int(self.current_count as i64),
        );
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}
