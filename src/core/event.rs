use super::types::ComponentId;

pub type EventType = String;
pub type EventId = String;

#[derive(Debug, Clone)]
pub struct Event {
    pub id: EventId,
    pub event_type: EventType,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
}

impl Event {
    pub fn new(
        id: EventId,
        event_type: EventType,
        source_id: ComponentId,
        target_ids: Option<Vec<ComponentId>>,
    ) -> Self {
        Self {
            id,
            event_type,
            source_id,
            target_ids,
        }
    }
}