use crate::core::components::types::PortType;
use crate::core::components::module::{ProcessorModule, MemoryModule};
use crate::core::components::state::MemoryData;

/// Parametric processor reaction trait for component logic
/// 
/// This trait allows components to implement their business logic with optional context:
/// - `React` (default): Simple components that only need their own fields
/// - `React<&mut SimulationContext>`: Advanced components needing memory/timing access
pub trait React<Ctx = ()> {
    type Output;
    
    /// Execute the component's logic and return optional output
    fn react(&mut self, ctx: Ctx) -> Option<Self::Output>;
}

/// Memory update trait for stateful components
/// 
/// Memory components implement this trait to update their internal state each cycle.
/// The cycle method is called once per simulation cycle after all processors have run.
pub trait Cycle {
    type Output;
    
    /// Update internal state and return optional output for next cycle
    fn cycle(&mut self) -> Option<Self::Output>;
}

/// Manual component definition trait for processors
/// 
/// Components implementing this trait provide explicit port definitions and module creation.
/// This eliminates meta-programming and provides full control over component behavior.
pub trait Component {
    /// Define all ports for this component type
    /// 
    /// Returns a vector of (port_name, port_type) pairs that explicitly define
    /// the component's interface. Port names must match struct field names.
    fn define_ports() -> Vec<(String, PortType)>;
    
    /// Convert this component type into a processor module
    /// 
    /// This method creates the bridge between the component's React implementation
    /// and the simulation engine. It manually extracts inputs, creates the component
    /// instance, calls react(), and handles outputs.
    fn into_module() -> ProcessorModule;
}

/// Manual memory component definition trait
/// 
/// Memory components combine the Cycle trait for state updates with explicit
/// port definitions. They must have exactly one input and one output port,
/// and cannot have memory ports.
pub trait MemoryComponent: Cycle {
    /// Define all ports for this memory component type
    /// 
    /// Memory components MUST have exactly one input and one output port,
    /// and CANNOT have memory ports. These constraints are enforced by the
    /// `into_memory_module()` method.
    fn define_ports() -> Vec<(String, PortType)>;
    
    /// Convert this memory component type into a memory module
    /// 
    /// Creates the memory module that handles the component's state and
    /// integrates with the simulation engine's memory system.
    /// 
    /// # Validation
    /// 
    /// This method validates that the memory component has exactly one input port,
    /// exactly one output port, and no memory ports, as required by the architectural constraints.
    fn into_memory_module() -> MemoryModule<Self>
    where
        Self: Sized + MemoryData
    {
        let ports = Self::define_ports();
        
        // Count input, output, and memory ports
        let input_count = ports.iter().filter(|(_, port_type)| *port_type == PortType::Input).count();
        let output_count = ports.iter().filter(|(_, port_type)| *port_type == PortType::Output).count();
        let memory_count = ports.iter().filter(|(_, port_type)| *port_type == PortType::Memory).count();
        
        // Validate constraints
        if input_count != 1 {
            panic!("Memory component {} must have exactly one input port, found {}", 
                   std::any::type_name::<Self>(), input_count);
        }
        
        if output_count != 1 {
            panic!("Memory component {} must have exactly one output port, found {}", 
                   std::any::type_name::<Self>(), output_count);
        }
        
        if memory_count != 0 {
            panic!("Memory component {} cannot have memory ports, found {}", 
                   std::any::type_name::<Self>(), memory_count);
        }
        
        // If validation passes, create the memory module
        MemoryModule::new(&format!("memory_{}", std::any::type_name::<Self>()))
    }
}

/// Helper trait for component utilities
/// 
/// This trait provides helper methods for component types.
pub trait ReactHelper {
    /// Get the component type name for debugging and registration
    fn component_type_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Marker trait for components that can be used in the simulation
/// 
/// This trait is automatically implemented for all types that implement
/// either Component or MemoryComponent.
pub trait SimulationComponent {
    /// Get the component type name for debugging and registration
    fn component_type_name() -> &'static str;
}

impl<T> SimulationComponent for T
where
    T: Component,
{
    fn component_type_name() -> &'static str {
        std::any::type_name::<T>()
    }
}