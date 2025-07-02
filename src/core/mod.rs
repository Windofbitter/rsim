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

// Re-export all commonly used types for backward compatibility
pub use values::{Event, TypedValue, TypedData, TypedInputs, TypedOutputs, EventInputs, EventOutputs, TypedInputMap, TypedOutputMap, EventInputMap, EventOutputMap};
pub use components::{ComponentManager, ComponentRegistry, ComponentType, ComponentModule, ProcessorModule, MemoryModule, PortSpec, PortType, EvaluationContext, LegacyEvaluationContext, TypeSafeMemoryProxy, ComponentState, MemoryData};
pub use connections::{ConnectionManager, ConnectionValidator, PortValidator};
pub use memory::{TypeSafeCentralMemoryProxy, MemoryError};
pub use execution::{CycleEngine, ExecutionOrderBuilder, SimulationEngine};
pub use builder::{Simulation, SimulationExt};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_events;

#[cfg(test)]
mod event_usage_example;