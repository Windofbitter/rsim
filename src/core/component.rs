use super::types::{ComponentId, ComponentValue};
use super::component_manager::ComponentInstance;
use super::state::ComponentState;

// Use existing ComponentValue for type consistency
pub type Event = ComponentValue;

/// Unified component structure that represents module-based components only
pub struct Component {
    /// Component instance from the new system
    pub instance: ComponentInstance,
}

impl Component {
    /// Create a new module-based component
    pub fn new(instance: ComponentInstance) -> Self {
        Self { instance }
    }

    /// Get the component ID
    pub fn id(&self) -> &ComponentId {
        self.instance.id()
    }

    /// Get the module name
    pub fn module_name(&self) -> &str {
        self.instance.module_name()
    }

    /// Check if this is a processing component
    pub fn is_processing(&self) -> bool {
        self.instance.is_processing()
    }

    /// Check if this is a memory component
    pub fn is_memory(&self) -> bool {
        self.instance.is_memory()
    }

    /// Check if this is a probe component
    pub fn is_probe(&self) -> bool {
        self.instance.is_probe()
    }

    /// Get mutable access to component state
    pub fn state_mut(&mut self) -> Option<&mut dyn ComponentState> {
        self.instance.state_mut()
    }

    /// Get immutable access to component state
    pub fn state(&self) -> Option<&dyn ComponentState> {
        self.instance.state()
    }

    /// Get a reference to the underlying component instance
    pub fn instance(&self) -> &ComponentInstance {
        &self.instance
    }

    /// Get a mutable reference to the underlying component instance
    pub fn instance_mut(&mut self) -> &mut ComponentInstance {
        &mut self.instance
    }
}

#[derive(Debug, Clone)]
pub enum MemoryError {
    InvalidAddress(String),
    InvalidPort(String),
    MemoryNotFound(ComponentId),
    OperationFailed(String),
    TypeMismatch(String),
}