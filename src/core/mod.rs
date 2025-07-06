// Organized module structure
pub mod values;
pub mod components;
pub mod connections;
pub mod memory;
pub mod execution;
pub mod builder;

// Core types (keep at root level)
pub mod types;
pub use types::{ComponentId, OutputPort, InputPort, MemoryPort};

// Re-export all commonly used types
pub use values::{Event, TypedValue, TypedData, UnifiedInputMap, UnifiedOutputMap};
pub use components::{
    ProcessorModule, MemoryModule, ModuleTrait, MemoryStats,
    ComponentState, MemoryData,
    React, Cycle, Component, MemoryComponent, ReactHelper, SimulationComponent,
    PortType, SimulationContext, Inputs, Outputs, ComponentError
};
pub use connections::{ConnectionManager, ConnectionValidator, PortValidator};
pub use memory::{MemoryProxy, MemoryError};
pub use execution::{CycleEngine, SimulationEngine, SimulationConfig, ConcurrencyMode};
pub use builder::{Simulation, SimulationExt};