use crate::core::components::state::{ComponentState, MemoryData};
use crate::core::types::ComponentId;
use crate::core::values::implementations::{TypedInputMap, EventInputMap};

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