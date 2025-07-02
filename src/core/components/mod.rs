pub mod manager;
pub mod registry;
pub mod module;
pub mod state;

// Re-export commonly used types
pub use manager::ComponentManager;
pub use registry::{ComponentRegistry, ComponentType};
pub use module::{ComponentModule, ProcessorModule, MemoryModule, PortSpec, PortType, EvaluationContext, LegacyEvaluationContext, TypeSafeMemoryProxy};
pub use state::{ComponentState, MemoryData};