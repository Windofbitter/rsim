use rsim::*;

/// Structured state memory for Baker components
/// Stores all internal state data in a cohesive structure
#[derive(Clone, Debug)]
pub struct BakerState {
    /// Remaining cycles before baker can produce next bread
    pub remaining_cycles: i64,
    /// Total bread produced by this baker
    pub total_produced: i64,
    /// RNG state for deterministic random timing
    pub rng_state: i64,
}

impl BakerState {
    /// Create a new BakerState with default values
    pub fn new() -> Self {
        Self {
            remaining_cycles: 0,
            total_produced: 0,
            rng_state: 54321,
        }
    }
}

// Implement MemoryData trait so BakerState can be stored in memory components
impl rsim::core::components::state::MemoryData for BakerState {}

impl Cycle for BakerState {
    type Output = i64;
    
    fn cycle(&mut self) -> Option<Self::Output> {
        // Return the current remaining cycles as output for debugging
        Some(self.remaining_cycles)
    }
}

// Implement MemoryComponent trait for BakerState using macro
impl_memory_component!(BakerState, {
    input: input,
    output: output
});

/// Structured state memory for Fryer components
/// Stores all internal state data in a cohesive structure
#[derive(Clone, Debug)]
pub struct FryerState {
    /// Remaining cycles before fryer can produce next meat
    pub remaining_cycles: i64,
    /// Total meat produced by this fryer
    pub total_produced: i64,
    /// RNG state for deterministic random timing
    pub rng_state: i64,
}

impl FryerState {
    /// Create a new FryerState with default values
    pub fn new() -> Self {
        Self {
            remaining_cycles: 0,
            total_produced: 0,
            rng_state: 12345,
        }
    }
}

// Implement MemoryData trait so FryerState can be stored in memory components
impl rsim::core::components::state::MemoryData for FryerState {}

impl Cycle for FryerState {
    type Output = i64;
    
    fn cycle(&mut self) -> Option<Self::Output> {
        // Return the current remaining cycles as output for debugging
        Some(self.remaining_cycles)
    }
}

// Implement MemoryComponent trait for FryerState using macro
impl_memory_component!(FryerState, {
    input: input,
    output: output
});

/// Structured state memory for Assembler components
/// Stores all internal state data in a cohesive structure
#[derive(Clone, Debug)]
pub struct AssemblerState {
    /// Remaining cycles before assembler can complete current burger
    pub remaining_cycles: i64,
    /// Total burgers assembled by this assembler
    pub total_assembled: i64,
    /// RNG state for deterministic random timing
    pub rng_state: i64,
}

impl AssemblerState {
    /// Create a new AssemblerState with default values
    pub fn new() -> Self {
        Self {
            remaining_cycles: 0,
            total_assembled: 0,
            rng_state: 98765,
        }
    }
}

// Implement MemoryData trait so AssemblerState can be stored in memory components
impl rsim::core::components::state::MemoryData for AssemblerState {}

impl Cycle for AssemblerState {
    type Output = i64;
    
    fn cycle(&mut self) -> Option<Self::Output> {
        // Return the current remaining cycles as output for debugging
        Some(self.remaining_cycles)
    }
}

// Implement MemoryComponent trait for AssemblerState using macro
impl_memory_component!(AssemblerState, {
    input: input,
    output: output
});

/// Structured state memory for Customer components
/// Stores all internal state data in a cohesive structure
#[derive(Clone, Debug)]
pub struct CustomerState {
    /// Remaining cycles before customer finishes consuming current burger
    pub remaining_cycles: i64,
    /// Total burgers consumed by this customer
    pub total_consumed: i64,
    /// RNG state for deterministic random timing
    pub rng_state: i64,
}

impl CustomerState {
    /// Create a new CustomerState with default values
    pub fn new() -> Self {
        Self {
            remaining_cycles: 0,
            total_consumed: 0,
            rng_state: 11111,
        }
    }
}

// Implement MemoryData trait so CustomerState can be stored in memory components
impl rsim::core::components::state::MemoryData for CustomerState {}

impl Cycle for CustomerState {
    type Output = i64;
    
    fn cycle(&mut self) -> Option<Self::Output> {
        // Return the current remaining cycles as output for debugging
        Some(self.remaining_cycles)
    }
}

// Implement MemoryComponent trait for CustomerState using macro
impl_memory_component!(CustomerState, {
    input: input,
    output: output
});