// Re-export commonly used types from new module structure
pub use super::processor_module::ProcessorModule;
pub use super::memory_module::{MemoryModule, MemoryModuleTrait};
pub use super::evaluation_context::{EvaluationContext, LegacyEvaluationContext, TypeSafeMemoryProxy};
pub use super::port_specs::{PortSpec, PortType};
pub use super::memory_stats::MemoryStats;

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

