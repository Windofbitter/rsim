/// Port specification for component inputs, outputs, and memory ports
#[derive(Debug, Clone)]
pub struct PortSpec {
    /// Port name
    pub name: String,
    /// Port type (input, output, memory)
    pub port_type: PortType,
    /// Whether this port is required for component operation
    pub required: bool,
    /// Optional description for documentation
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortType {
    Input,
    Output,
    Memory,
}

impl PortSpec {
    /// Create a new required input port
    pub fn input(name: &str) -> Self {
        Self {
            name: name.to_string(),
            port_type: PortType::Input,
            required: true,
            description: None,
        }
    }

    /// Create a new optional input port
    pub fn input_optional(name: &str) -> Self {
        Self {
            name: name.to_string(),
            port_type: PortType::Input,
            required: false,
            description: None,
        }
    }

    /// Create a new output port
    pub fn output(name: &str) -> Self {
        Self {
            name: name.to_string(),
            port_type: PortType::Output,
            required: false, // outputs are not "required" in the same sense
            description: None,
        }
    }

    /// Create a new memory port
    pub fn memory(name: &str) -> Self {
        Self {
            name: name.to_string(),
            port_type: PortType::Memory,
            required: false, // memory ports are optional
            description: None,
        }
    }

    /// Add a description to this port
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Mark this port as optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Mark this port as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}