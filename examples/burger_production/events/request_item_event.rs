use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RequestItemEvent {
    id: EventId,
    source_id: ComponentId,
    target_ids: Option<Vec<ComponentId>>,
    requester_id: String,
    item_type: String,
}

impl RequestItemEvent {
    pub fn new(
        source_id: ComponentId,
        target_ids: Option<Vec<ComponentId>>,
        requester_id: String,
        item_type: String,
    ) -> Self {
        Self {
            id: format!("request_item_{}", uuid::Uuid::new_v4()),
            source_id,
            target_ids,
            requester_id,
            item_type,
        }
    }
}

impl Event for RequestItemEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        "RequestItemEvent"
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
            "requester_id".to_string(),
            ComponentValue::String(self.requester_id.clone()),
        );
        data.insert(
            "item_type".to_string(),
            ComponentValue::String(self.item_type.clone()),
        );
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}
