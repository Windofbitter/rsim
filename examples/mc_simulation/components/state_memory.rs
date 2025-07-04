use rsim::*;

/// Simple state memory component for storing individual values
/// Used for storing component internal state like timers, counters, etc.
#[derive(Clone, Debug)]
pub struct StateMemory {
    /// A dummy field - the actual storage is handled by the memory system
    pub _dummy: i64,
}

impl StateMemory {
    /// Create a new StateMemory
    pub fn new() -> Self {
        Self {
            _dummy: 0,
        }
    }
}

// Implement MemoryData trait so StateMemory can be stored in memory components
impl rsim::core::components::state::MemoryData for StateMemory {}

impl Cycle for StateMemory {
    type Output = i64;
    
    fn cycle(&mut self) -> Option<Self::Output> {
        // For state memory, we don't need to output anything specific
        // Just return a dummy value
        Some(0)
    }
}

// Implement MemoryComponent trait for StateMemory using macro
impl_memory_component!(StateMemory, {
    input: input,
    output: output
});