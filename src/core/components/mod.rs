pub mod module;
pub mod state;
pub mod traits;
pub mod types;

// Re-export commonly used types
pub use module::{ProcessorModule, MemoryModule, ModuleTrait, MemoryStats};
pub use state::{ComponentState, MemoryData};
pub use traits::{React, Cycle, Component, MemoryComponent, ReactHelper, SimulationComponent};
pub use types::{PortType, SimulationContext, Inputs, InputsExt, Outputs, OutputsExt, ComponentError};