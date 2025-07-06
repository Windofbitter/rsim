pub mod module;
pub mod state;
pub mod traits;
pub mod types;
pub mod processor_module;
pub mod memory_module;
pub mod evaluation_context;
pub mod port_specs;
pub mod memory_stats;

// Re-export commonly used types
pub use module::{ProcessorModule, MemoryModule, ModuleTrait, MemoryStats};
pub use state::{ComponentState, MemoryData};
pub use traits::{React, Cycle, Component, MemoryComponent, ReactHelper, SimulationComponent};
pub use types::{PortType, SimulationContext, Inputs, InputsExt, Outputs, OutputsExt, ComponentError};