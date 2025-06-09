pub type ComponentId = String;
pub type EventType = String;

#[derive(Debug, Clone)]
pub enum ComponentValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}



#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
}

impl Event {
    pub fn new(
        event_type: EventType,
        source_id: ComponentId,
        target_ids: Option<Vec<ComponentId>>,
    ) -> Self {
        Self {
            event_type,
            source_id,
            target_ids,
        }
    }
}

pub trait BaseComponent {
    fn component_id(&self) -> &ComponentId;
    fn subscriptions(&self) -> &[EventType];
    fn react_atomic(&mut self, events: Vec<Event>) -> Vec<(Event, u64)>;
}

