use crate::core::types::ComponentId;
use crate::core::memory::proxy::MemoryProxy;
use crate::core::components::state::MemoryData;

/// Port type enumeration for component interfaces
/// 
/// This simplified enum removes the complexity of PortSpec builders
/// and provides a clear, explicit way to define component ports.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PortType {
    /// Input port - receives data from other components
    Input,
    /// Output port - sends data to other components  
    Output,
    /// Memory port - connects to memory components for read/write access
    Memory,
}

impl PortType {
    /// Check if this port type can be connected to another port type
    pub fn can_connect_to(&self, other: &PortType) -> bool {
        match (self, other) {
            (PortType::Output, PortType::Input) => true,
            (PortType::Memory, PortType::Memory) => true,
            _ => false,
        }
    }
    
    /// Get a human-readable description of this port type
    pub fn description(&self) -> &'static str {
        match self {
            PortType::Input => "Input port that receives data from other components",
            PortType::Output => "Output port that sends data to other components",
            PortType::Memory => "Memory port that connects to memory components",
        }
    }
}

/// Unified simulation context for component evaluation
/// 
/// This replaces the dual context system (EvaluationContext vs LegacyEvaluationContext)
/// with a single, clean interface that supports progressive disclosure.
pub struct SimulationContext<'a> {
    /// Input values from connected components
    pub inputs: &'a dyn Inputs,
    /// Memory proxy for type-safe memory access
    pub memory: &'a mut MemoryProxy<'a>,
    /// Component ID for context and debugging
    pub component_id: &'a ComponentId,
}

impl<'a> SimulationContext<'a> {
    /// Create a new simulation context
    pub fn new(
        inputs: &'a dyn Inputs,
        memory: &'a mut MemoryProxy<'a>,
        component_id: &'a ComponentId,
    ) -> Self {
        Self {
            inputs,
            memory,
            component_id,
        }
    }
    
    /// Get the component's unique identifier
    pub fn component_id(&self) -> &ComponentId {
        self.component_id
    }
    
    /// Get typed input value (convenience method)
    pub fn get_input<T: 'static + Clone>(&self, port: &str) -> Result<T, String> {
        self.inputs.get(port)
    }
    
    /// Get input timestamp (convenience method)
    pub fn get_input_timestamp(&self, port: &str) -> Result<u64, String> {
        self.inputs.get_timestamp(port)
    }
    
    /// Read from memory (convenience method)
    pub fn memory_read<T: MemoryData>(&self, port: &str, address: &str) -> Result<Option<T>, String> {
        self.memory.read(port, address)
    }
    
    /// Write to memory (convenience method)
    pub fn memory_write<T: MemoryData>(&mut self, port: &str, address: &str, data: T) -> Result<(), String> {
        self.memory.write(port, address, data)
    }
}

/// Unified input trait that merges TypedInputs and EventInputs
/// 
/// This trait provides progressive disclosure using type erasure for dyn compatibility.
pub trait Inputs {
    /// Get typed input value using type erasure
    fn get_typed_value(&self, port: &str) -> Result<crate::core::values::typed_value::TypedValue, String>;
    
    /// Get timestamp for input (for timing-aware components)
    fn get_timestamp(&self, port: &str) -> Result<u64, String>;
    
    /// Check if input port exists
    fn has_input(&self, port: &str) -> bool;
    
    /// Get all available input port names
    fn input_ports(&self) -> Vec<&str>;
    
    /// Get the number of inputs
    fn len(&self) -> usize;
    
    /// Check if there are no inputs
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Extension trait for convenient generic getting of inputs
pub trait InputsExt {
    /// Get typed input value (convenience method)
    fn get<T: 'static + Clone>(&self, port: &str) -> Result<T, String>;
}

impl<I: Inputs + ?Sized> InputsExt for I {
    fn get<T: 'static + Clone>(&self, port: &str) -> Result<T, String> {
        let typed_value = self.get_typed_value(port)?;
        let value: &T = typed_value.get()?;
        Ok(value.clone())
    }
}

/// Unified output trait that merges TypedOutputs and EventOutputs
/// 
/// This trait provides a simple interface for setting component outputs
/// while supporting both simple values and timed events.
/// 
/// Note: This trait uses type erasure to be dyn-compatible.
pub trait Outputs {
    /// Set typed output value using type erasure
    fn set_typed_value(&mut self, port: &str, value: crate::core::values::typed_value::TypedValue) -> Result<(), String>;
    
    /// Set output with timestamp using type erasure
    fn set_typed_value_with_timestamp(
        &mut self, 
        port: &str, 
        value: crate::core::values::typed_value::TypedValue, 
        timestamp: u64
    ) -> Result<(), String>;
    
    /// Check if output port is valid
    fn is_valid_port(&self, port: &str) -> bool;
    
    /// Get all expected output port names
    fn expected_ports(&self) -> Vec<&str>;
}

/// Extension trait for convenient generic setting of outputs
pub trait OutputsExt {
    /// Set typed output value (convenience method)
    fn set<T: Send + Sync + Clone + 'static>(&mut self, port: &str, value: T) -> Result<(), String>;
    
    /// Set output with timestamp (convenience method)
    fn set_with_timestamp<T: Send + Sync + Clone + 'static>(
        &mut self, 
        port: &str, 
        value: T, 
        timestamp: u64
    ) -> Result<(), String>;
}

impl<O: Outputs + ?Sized> OutputsExt for O {
    fn set<T: Send + Sync + Clone + 'static>(&mut self, port: &str, value: T) -> Result<(), String> {
        let typed_value = crate::core::values::typed_value::TypedValue::new(value);
        self.set_typed_value(port, typed_value)
    }
    
    fn set_with_timestamp<T: Send + Sync + Clone + 'static>(
        &mut self, 
        port: &str, 
        value: T, 
        timestamp: u64
    ) -> Result<(), String> {
        let typed_value = crate::core::values::typed_value::TypedValue::new(value);
        self.set_typed_value_with_timestamp(port, typed_value, timestamp)
    }
}

/// Error types for component operations
#[derive(Debug, Clone)]
pub enum ComponentError {
    /// Port not found
    PortNotFound(String),
    /// Invalid port type
    InvalidPortType(String),
    /// Type mismatch
    TypeMismatch(String),
    /// Component not found
    ComponentNotFound(String),
    /// Invalid connection
    InvalidConnection(String),
    /// Memory access error
    MemoryError(String),
}

impl std::fmt::Display for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentError::PortNotFound(msg) => write!(f, "Port not found: {}", msg),
            ComponentError::InvalidPortType(msg) => write!(f, "Invalid port type: {}", msg),
            ComponentError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            ComponentError::ComponentNotFound(msg) => write!(f, "Component not found: {}", msg),
            ComponentError::InvalidConnection(msg) => write!(f, "Invalid connection: {}", msg),
            ComponentError::MemoryError(msg) => write!(f, "Memory error: {}", msg),
        }
    }
}

impl std::error::Error for ComponentError {}

/// Convert string errors to ComponentError
impl From<String> for ComponentError {
    fn from(msg: String) -> Self {
        ComponentError::ComponentNotFound(msg)
    }
}

/// Convert &str errors to ComponentError
impl From<&str> for ComponentError {
    fn from(msg: &str) -> Self {
        ComponentError::ComponentNotFound(msg.to_string())
    }
}