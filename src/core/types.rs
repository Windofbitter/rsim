/// Enhanced component identifier with module type information
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentId {
    pub(crate) id: String,
    pub(crate) module_type: String,
}

impl ComponentId {
    /// Create a new component ID
    pub fn new(id: String, module_type: String) -> Self {
        Self { id, module_type }
    }
    
    /// Get the raw ID string
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Get the module type
    pub fn module_type(&self) -> &str {
        &self.module_type
    }
    
    /// Create an output port handle
    pub fn output(&self, port: &str) -> OutputPort {
        OutputPort {
            component_id: self.clone(),
            port_name: port.to_string(),
        }
    }
    
    /// Create an input port handle
    pub fn input(&self, port: &str) -> InputPort {
        InputPort {
            component_id: self.clone(),
            port_name: port.to_string(),
        }
    }
    
    /// Create a memory port handle
    pub fn memory_port(&self, port: &str) -> MemoryPort {
        MemoryPort {
            component_id: self.clone(),
            port_name: port.to_string(),
        }
    }
}

impl std::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Handle for an output port
#[derive(Debug, Clone)]
pub struct OutputPort {
    pub(crate) component_id: ComponentId,
    pub(crate) port_name: String,
}

impl OutputPort {
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }
    
    pub fn port_name(&self) -> &str {
        &self.port_name
    }
}

/// Handle for an input port
#[derive(Debug, Clone)]
pub struct InputPort {
    pub(crate) component_id: ComponentId,
    pub(crate) port_name: String,
}

impl InputPort {
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }
    
    pub fn port_name(&self) -> &str {
        &self.port_name
    }
}

/// Handle for a memory port
#[derive(Debug, Clone)]
pub struct MemoryPort {
    pub(crate) component_id: ComponentId,
    pub(crate) port_name: String,
}

impl MemoryPort {
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }
    
    pub fn port_name(&self) -> &str {
        &self.port_name
    }
}
