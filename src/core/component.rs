use super::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

// Use existing ComponentValue for type consistency
pub type Event = ComponentValue;

// Base trait for all components
pub trait BaseComponent: Send {
    fn component_id(&self) -> &ComponentId;
}

#[derive(Debug, Clone)]
pub enum MemoryError {
    InvalidAddress(String),
    InvalidPort(String),
    MemoryNotFound(ComponentId),
    OperationFailed(String),
}

// Engine-level memory proxy interface - centralized memory access
pub trait EngineMemoryProxy {
    fn read(&self, component_id: &ComponentId, port: &str, address: &str) -> Result<Option<Event>, MemoryError>;
    fn write(&mut self, component_id: &ComponentId, port: &str, address: &str, data: Event) -> Result<(), MemoryError>;
}

// Stateless processing components
pub trait ProcessingComponent: BaseComponent {
    fn input_ports(&self) -> Vec<&'static str>;
    fn output_ports(&self) -> Vec<&'static str>;
    fn memory_ports(&self) -> Vec<&'static str> {
        vec![]
    }

    // Evaluate with access to engine memory proxy
    fn evaluate(
        &self,
        inputs: &HashMap<String, Event>,
        memory_proxy: &mut dyn EngineMemoryProxy,
    ) -> HashMap<String, Event>;
}

// Stateful memory components
pub trait MemoryComponent: BaseComponent {
    fn memory_id(&self) -> &str;
    fn input_port(&self) -> &'static str {
        "in"
    }
    fn output_port(&self) -> &'static str {
        "out"
    }

    // Read from previous cycle's state snapshot
    fn read_snapshot(&self, address: &str) -> Option<Event>;

    // Write operation (applied immediately to current state)
    fn write(&mut self, address: &str, data: Event) -> bool;

    // Called at end of cycle to create snapshot for next cycle
    fn end_cycle(&mut self);
}

// Passive monitoring components
pub trait ProbeComponent: BaseComponent {
    fn probe(&mut self, source: &ComponentId, port: &str, event: &Event);
}
