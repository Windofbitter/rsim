use crate::core::components::state::{ComponentState, MemoryData};
use crate::core::types::ComponentId;
use crate::core::values::implementations::{TypedInputMap, EventInputMap, EventOutputMap};
use std::collections::HashMap;

/// Evaluation context provided to component modules during evaluation.
/// Contains inputs, memory access, and output collection.
pub struct EvaluationContext<'a> {
    /// Event input values from connected components
    pub inputs: &'a EventInputMap,
    /// Memory proxy for type-safe memory access
    pub memory: &'a mut crate::core::memory::proxy::MemoryProxy<'a>,
    /// Component's current state (if any)
    pub state: Option<&'a mut dyn ComponentState>,
    /// Component ID for context
    pub component_id: &'a ComponentId,
}

/// Legacy evaluation context for backward compatibility
pub struct LegacyEvaluationContext<'a> {
    /// Typed input values from connected components
    pub inputs: &'a TypedInputMap,
    /// Memory proxy for type-safe memory access
    pub memory: &'a mut crate::core::memory::proxy::MemoryProxy<'a>,
    /// Component's current state (if any)
    pub state: Option<&'a mut dyn ComponentState>,
    /// Component ID for context
    pub component_id: &'a ComponentId,
}

/// Type-safe memory proxy trait for new component system
pub trait TypeSafeMemoryProxy {
    /// Read typed data from memory
    fn read<T: MemoryData>(&self, port: &str, address: &str) -> Result<Option<T>, String>;
    /// Write typed data to memory
    fn write<T: MemoryData>(&mut self, port: &str, address: &str, data: T) -> Result<(), String>;
}

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

/// Processing component module that defines stateless computation
#[derive(Clone)]
pub struct ProcessorModule {
    /// Component name/type
    pub name: String,
    /// Input port specifications
    pub input_ports: Vec<PortSpec>,
    /// Output port specifications
    pub output_ports: Vec<PortSpec>,
    /// Memory port specifications
    pub memory_ports: Vec<PortSpec>,
    /// Evaluation function with event outputs
    pub evaluate_fn: fn(&mut EvaluationContext, &mut EventOutputMap) -> Result<(), String>,
}

impl ProcessorModule {
    /// Create a new processor module with validation
    pub fn new(
        name: &str,
        input_ports: Vec<PortSpec>,
        output_ports: Vec<PortSpec>,
        memory_ports: Vec<PortSpec>,
        evaluate_fn: fn(&mut EvaluationContext, &mut EventOutputMap) -> Result<(), String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            input_ports,
            output_ports,
            memory_ports,
            evaluate_fn,
        }
    }

    /// Validate that this processing module meets architecture constraints
    /// Processing components should have multiple ports but single output type
    pub fn validate_architecture(&self) -> Result<(), String> {
        // Validate component has a name
        if self.name.is_empty() {
            return Err("Processing module must have a valid name".to_string());
        }

        // Ensure processing components are stateless (enforced by design - no state field)
        // Ensure output wrapping (enforced by EventOutputMap in evaluate_fn signature)

        // Note: Single output type constraint is enforced by the evaluate_fn signature
        // and EventOutputMap which wraps all outputs in Event structures
        
        Ok(())
    }

    /// Get all input port names
    pub fn input_port_names(&self) -> Vec<&str> {
        self.input_ports.iter().map(|p| p.name.as_str()).collect()
    }

    /// Get all output port names
    pub fn output_port_names(&self) -> Vec<&str> {
        self.output_ports.iter().map(|p| p.name.as_str()).collect()
    }

    /// Get all memory port names
    pub fn memory_port_names(&self) -> Vec<&str> {
        self.memory_ports.iter().map(|p| p.name.as_str()).collect()
    }

    /// Check if an input port exists
    pub fn has_input_port(&self, name: &str) -> bool {
        self.input_ports.iter().any(|p| p.name == name)
    }

    /// Check if an output port exists
    pub fn has_output_port(&self, name: &str) -> bool {
        self.output_ports.iter().any(|p| p.name == name)
    }

    /// Check if a memory port exists
    pub fn has_memory_port(&self, name: &str) -> bool {
        self.memory_ports.iter().any(|p| p.name == name)
    }
}


/// Trait for memory modules that can store and retrieve typed data
pub trait MemoryModuleTrait: Send {
    /// Get the memory ID for this module
    fn memory_id(&self) -> &str;
    
    /// Read data from memory (type-erased)
    fn read_any(&self, address: &str) -> Option<Box<dyn std::any::Any + Send>>;
    
    /// Write data to memory (type-erased)
    fn write_any(&mut self, address: &str, data: Box<dyn std::any::Any + Send>) -> bool;
    
    /// Create a snapshot for next cycle
    fn create_snapshot(&mut self);
    
    /// Get a clone of this memory module
    fn clone_module(&self) -> Box<dyn MemoryModuleTrait>;
}

/// Concrete memory module implementation for specific data types
pub struct MemoryModule<T: MemoryData> {
    /// Memory identifier
    pub memory_id: String,
    /// Current state (gets written to during cycle)
    current_state: HashMap<String, T>,
    /// Snapshot from previous cycle (gets read from during cycle)
    snapshot: HashMap<String, T>,
}

impl<T: MemoryData> MemoryModule<T> {
    /// Create a new memory module with validation
    pub fn new(memory_id: &str) -> Self {
        Self {
            memory_id: memory_id.to_string(),
            current_state: HashMap::new(),
            snapshot: HashMap::new(),
        }
    }

    /// Validate that this memory module has the correct architecture constraints
    /// Memory modules should have single input/output and state management
    pub fn validate_architecture(&self) -> Result<(), String> {
        // Note: Memory modules don't have traditional ports like processing components
        // Their "ports" are the memory connection points managed by the proxy system
        // This validation ensures the memory module is properly configured
        if self.memory_id.is_empty() {
            return Err("Memory module must have a valid memory_id".to_string());
        }
        Ok(())
    }

    /// Read from snapshot (previous cycle data)
    pub fn read(&self, address: &str) -> Option<T> {
        self.snapshot.get(address).cloned()
    }

    /// Write to current state (affects next cycle)
    pub fn write(&mut self, address: &str, data: T) -> bool {
        self.current_state.insert(address.to_string(), data);
        true
    }
}

impl<T: MemoryData> MemoryModuleTrait for MemoryModule<T> {
    fn memory_id(&self) -> &str {
        &self.memory_id
    }

    fn read_any(&self, address: &str) -> Option<Box<dyn std::any::Any + Send>> {
        self.snapshot.get(address).map(|data| {
            let boxed: Box<dyn std::any::Any + Send> = Box::new(data.clone());
            boxed
        })
    }

    fn write_any(&mut self, address: &str, data: Box<dyn std::any::Any + Send>) -> bool {
        if let Ok(typed_data) = data.downcast::<T>() {
            self.current_state.insert(address.to_string(), *typed_data);
            true
        } else {
            false
        }
    }

    fn create_snapshot(&mut self) {
        self.snapshot = self.current_state.clone();
    }

    fn clone_module(&self) -> Box<dyn MemoryModuleTrait> {
        Box::new(MemoryModule {
            memory_id: self.memory_id.clone(),
            current_state: self.current_state.clone(),
            snapshot: self.snapshot.clone(),
        })
    }
}

/// Enum representing different types of component modules
pub enum ComponentModule {
    Processing(ProcessorModule),
    Memory(Box<dyn MemoryModuleTrait>),
}

impl Clone for ComponentModule {
    fn clone(&self) -> Self {
        match self {
            ComponentModule::Processing(proc_module) => ComponentModule::Processing(proc_module.clone()),
            ComponentModule::Memory(memory_module) => ComponentModule::Memory(memory_module.clone_module()),
        }
    }
}

impl ComponentModule {
    /// Get the name of this component module
    pub fn name(&self) -> &str {
        match self {
            ComponentModule::Processing(module) => &module.name,
            ComponentModule::Memory(module) => module.memory_id(),
        }
    }

    /// Check if this is a processing module
    pub fn is_processing(&self) -> bool {
        matches!(self, ComponentModule::Processing(_))
    }

    /// Check if this is a memory module
    pub fn is_memory(&self) -> bool {
        matches!(self, ComponentModule::Memory(_))
    }

    /// Get as processing module
    pub fn as_processing(&self) -> Option<&ProcessorModule> {
        match self {
            ComponentModule::Processing(module) => Some(module),
            _ => None,
        }
    }

    /// Get as memory module
    pub fn as_memory(&self) -> Option<&dyn MemoryModuleTrait> {
        match self {
            ComponentModule::Memory(module) => Some(module.as_ref()),
            _ => None,
        }
    }
    
    /// Get as mutable memory module
    pub fn as_memory_mut(&mut self) -> Option<&mut dyn MemoryModuleTrait> {
        match self {
            ComponentModule::Memory(module) => Some(module.as_mut()),
            _ => None,
        }
    }

    /// Get all ports for this component module
    pub fn ports(&self) -> Vec<(String, crate::core::components::types::PortType)> {
        match self {
            ComponentModule::Processing(module) => {
                let mut ports = Vec::new();
                
                // Add input ports
                for port in &module.input_ports {
                    ports.push((port.name.clone(), crate::core::components::types::PortType::Input));
                }
                
                // Add output ports
                for port in &module.output_ports {
                    ports.push((port.name.clone(), crate::core::components::types::PortType::Output));
                }
                
                // Add memory ports
                for port in &module.memory_ports {
                    ports.push((port.name.clone(), crate::core::components::types::PortType::Memory));
                }
                
                ports
            }
            ComponentModule::Memory(_) => {
                // Memory modules don't have ports in the traditional sense
                // They are connected to via memory ports on other components
                Vec::new()
            }
        }
    }
}

/// Unified trait for all component modules
pub trait ModuleTrait: Send {
    /// Get the module name/identifier
    fn name(&self) -> &str;
    
    /// Clone this module
    fn clone_module(&self) -> Box<dyn ModuleTrait>;
}

/// Implementation of ModuleTrait for ComponentModule
impl ModuleTrait for ComponentModule {
    fn name(&self) -> &str {
        self.name()
    }
    
    fn clone_module(&self) -> Box<dyn ModuleTrait> {
        Box::new(self.clone())
    }
}

/// Memory statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Total number of memory addresses
    pub total_addresses: usize,
    /// Number of active (non-empty) addresses
    pub active_addresses: usize,
    /// Memory usage in bytes (estimated)
    pub memory_usage_bytes: usize,
    /// Number of read operations in current cycle
    pub read_count: usize,
    /// Number of write operations in current cycle
    pub write_count: usize,
}

impl MemoryStats {
    /// Create new empty memory stats
    pub fn new() -> Self {
        Self {
            total_addresses: 0,
            active_addresses: 0,
            memory_usage_bytes: 0,
            read_count: 0,
            write_count: 0,
        }
    }
    
    /// Reset cycle-specific counters
    pub fn reset_cycle_counters(&mut self) {
        self.read_count = 0;
        self.write_count = 0;
    }
    
    /// Update memory usage statistics
    pub fn update_usage(&mut self, total: usize, active: usize, bytes: usize) {
        self.total_addresses = total;
        self.active_addresses = active;
        self.memory_usage_bytes = bytes;
    }
    
    /// Increment read counter
    pub fn increment_reads(&mut self) {
        self.read_count += 1;
    }
    
    /// Increment write counter
    pub fn increment_writes(&mut self) {
        self.write_count += 1;
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self::new()
    }
}