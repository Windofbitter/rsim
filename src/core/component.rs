use std::collections::HashMap;

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
pub struct ComponentState {
    data: HashMap<String, ComponentValue>,
}

impl ComponentState {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: ComponentValue) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&ComponentValue> {
        self.data.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<ComponentValue> {
        self.data.remove(key)
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub data: ComponentValue,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
}

impl Event {
    pub fn new(
        event_type: EventType,
        data: ComponentValue,
        source_id: ComponentId,
        target_ids: Option<Vec<ComponentId>>,
    ) -> Self {
        Self {
            event_type,
            data,
            source_id,
            target_ids,
        }
    }
}

pub trait BaseComponent {
    fn component_id(&self) -> &ComponentId;
    fn state(&self) -> &ComponentState;
    fn state_mut(&mut self) -> &mut ComponentState;
    fn subscriptions(&self) -> &[EventType];
    fn react_atomic(&mut self, events: Vec<Event>) -> Vec<(Event, u64)>;
}