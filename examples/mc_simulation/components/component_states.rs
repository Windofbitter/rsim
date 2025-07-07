use rsim::*;
use crate::simulation_builder::{DelayMode, FixedDelayValues};

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
    /// Delay configuration for this baker
    pub delay_config: BakerDelayConfig,
}

/// Delay configuration for Baker components
#[derive(Clone, Debug)]
pub struct BakerDelayConfig {
    /// Delay mode (random or fixed)
    pub delay_mode: crate::simulation_builder::DelayMode,
    /// Minimum delay cycles (for random mode)
    pub min_delay: u32,
    /// Maximum delay cycles (for random mode)
    pub max_delay: u32,
    /// Fixed delay cycles (for fixed mode)
    pub fixed_delay: u32,
}

impl BakerState {
    /// Create a new BakerState with default values
    pub fn new() -> Self {
        Self {
            remaining_cycles: 0,
            total_produced: 0,
            rng_state: 54321,
            delay_config: BakerDelayConfig {
                delay_mode: crate::simulation_builder::DelayMode::Fixed,
                min_delay: 2,
                max_delay: 5,
                fixed_delay: 3,
            },
        }
    }
    
    /// Create a new BakerState with specific delay configuration
    pub fn with_delay_config(delay_config: BakerDelayConfig) -> Self {
        Self {
            remaining_cycles: 0,
            total_produced: 0,
            rng_state: 54321,
            delay_config,
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

/// Delay configuration for Fryer components
#[derive(Clone, Debug)]
pub struct FryerDelayConfig {
    /// Delay mode (random or fixed)
    pub delay_mode: crate::simulation_builder::DelayMode,
    /// Minimum delay cycles (for random mode)
    pub min_delay: u32,
    /// Maximum delay cycles (for random mode)
    pub max_delay: u32,
    /// Fixed delay cycles (for fixed mode)
    pub fixed_delay: u32,
}

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
    /// Delay configuration for this fryer
    pub delay_config: FryerDelayConfig,
}

impl FryerState {
    /// Create a new FryerState with default values
    pub fn new() -> Self {
        Self {
            remaining_cycles: 0,
            total_produced: 0,
            rng_state: 12345,
            delay_config: FryerDelayConfig {
                delay_mode: crate::simulation_builder::DelayMode::Fixed,
                min_delay: 3,
                max_delay: 7,
                fixed_delay: 5,
            },
        }
    }
    
    /// Create a new FryerState with specific delay configuration
    pub fn with_delay_config(delay_config: FryerDelayConfig) -> Self {
        Self {
            remaining_cycles: 0,
            total_produced: 0,
            rng_state: 12345,
            delay_config,
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

/// Delay configuration for Assembler components
#[derive(Clone, Debug)]
pub struct AssemblerDelayConfig {
    /// Delay mode (random or fixed)
    pub delay_mode: crate::simulation_builder::DelayMode,
    /// Minimum delay cycles (for random mode)
    pub min_delay: u32,
    /// Maximum delay cycles (for random mode)
    pub max_delay: u32,
    /// Fixed delay cycles (for fixed mode)
    pub fixed_delay: u32,
}

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
    /// Delay configuration for this assembler
    pub delay_config: AssemblerDelayConfig,
}

impl AssemblerState {
    /// Create a new AssemblerState with default values
    pub fn new() -> Self {
        Self {
            remaining_cycles: 0,
            total_assembled: 0,
            rng_state: 98765,
            delay_config: AssemblerDelayConfig {
                delay_mode: crate::simulation_builder::DelayMode::Fixed,
                min_delay: 1,
                max_delay: 3,
                fixed_delay: 2,
            },
        }
    }
    
    /// Create a new AssemblerState with specific delay configuration
    pub fn with_delay_config(delay_config: AssemblerDelayConfig) -> Self {
        Self {
            remaining_cycles: 0,
            total_assembled: 0,
            rng_state: 98765,
            delay_config,
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

/// Delay configuration for Customer components
#[derive(Clone, Debug)]
pub struct CustomerDelayConfig {
    /// Delay mode (random or fixed)
    pub delay_mode: crate::simulation_builder::DelayMode,
    /// Minimum delay cycles (for random mode)
    pub min_delay: u32,
    /// Maximum delay cycles (for random mode)
    pub max_delay: u32,
    /// Fixed delay cycles (for fixed mode)
    pub fixed_delay: u32,
}

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
    /// Delay configuration for this customer
    pub delay_config: CustomerDelayConfig,
}

impl CustomerState {
    /// Create a new CustomerState with default values
    pub fn new() -> Self {
        Self {
            remaining_cycles: 0,
            total_consumed: 0,
            rng_state: 11111,
            delay_config: CustomerDelayConfig {
                delay_mode: crate::simulation_builder::DelayMode::Fixed,
                min_delay: 1,
                max_delay: 5,
                fixed_delay: 3,
            },
        }
    }
    
    /// Create a new CustomerState with specific delay configuration
    pub fn with_delay_config(delay_config: CustomerDelayConfig) -> Self {
        Self {
            remaining_cycles: 0,
            total_consumed: 0,
            rng_state: 11111,
            delay_config,
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

/// Delay configuration memory component
/// Stores delay mode and fixed delay values for all components
#[derive(Clone, Debug)]
pub struct DelayConfig {
    /// Delay mode for all components (random or fixed)
    pub delay_mode: DelayMode,
    /// Fixed delay values when using DelayMode::Fixed
    pub fixed_delay_values: FixedDelayValues,
    /// Baker timing parameters (min, max cycles) for random mode
    pub baker_timing: (u32, u32),
    /// Fryer timing parameters (min, max cycles) for random mode
    pub fryer_timing: (u32, u32),
    /// Assembler timing parameters (min, max cycles) for random mode
    pub assembler_timing: (u32, u32),
    /// Customer timing parameters (min, max cycles) for random mode
    pub customer_timing: (u32, u32),
}

impl DelayConfig {
    /// Create a new DelayConfig with specified values
    pub fn new(
        delay_mode: DelayMode,
        fixed_delay_values: FixedDelayValues,
        baker_timing: (u32, u32),
        fryer_timing: (u32, u32),
        assembler_timing: (u32, u32),
        customer_timing: (u32, u32),
    ) -> Self {
        Self {
            delay_mode,
            fixed_delay_values,
            baker_timing,
            fryer_timing,
            assembler_timing,
            customer_timing,
        }
    }
    
    /// Get the delay for a baker component
    pub fn get_baker_delay(&self, rng_state: &mut i64) -> i64 {
        match self.delay_mode {
            DelayMode::Random => {
                use rand::{Rng, RngCore, SeedableRng};
                use rand::rngs::StdRng;
                let mut rng = StdRng::seed_from_u64(*rng_state as u64);
                let delay = rng.gen_range(self.baker_timing.0 as i64..=self.baker_timing.1 as i64);
                *rng_state = rng.next_u64() as i64;
                delay
            }
            DelayMode::Fixed => self.fixed_delay_values.baker_delay as i64,
        }
    }
    
    /// Get the delay for a fryer component
    pub fn get_fryer_delay(&self, rng_state: &mut i64) -> i64 {
        match self.delay_mode {
            DelayMode::Random => {
                use rand::{Rng, RngCore, SeedableRng};
                use rand::rngs::StdRng;
                let mut rng = StdRng::seed_from_u64(*rng_state as u64);
                let delay = rng.gen_range(self.fryer_timing.0 as i64..=self.fryer_timing.1 as i64);
                *rng_state = rng.next_u64() as i64;
                delay
            }
            DelayMode::Fixed => self.fixed_delay_values.fryer_delay as i64,
        }
    }
    
    /// Get the delay for an assembler component
    pub fn get_assembler_delay(&self, rng_state: &mut i64) -> i64 {
        match self.delay_mode {
            DelayMode::Random => {
                use rand::{Rng, RngCore, SeedableRng};
                use rand::rngs::StdRng;
                let mut rng = StdRng::seed_from_u64(*rng_state as u64);
                let delay = rng.gen_range(self.assembler_timing.0 as i64..=self.assembler_timing.1 as i64);
                *rng_state = rng.next_u64() as i64;
                delay
            }
            DelayMode::Fixed => self.fixed_delay_values.assembler_delay as i64,
        }
    }
    
    /// Get the delay for a customer component
    pub fn get_customer_delay(&self, rng_state: &mut i64) -> i64 {
        match self.delay_mode {
            DelayMode::Random => {
                use rand::{Rng, RngCore, SeedableRng};
                use rand::rngs::StdRng;
                let mut rng = StdRng::seed_from_u64(*rng_state as u64);
                let delay = rng.gen_range(self.customer_timing.0 as i64..=self.customer_timing.1 as i64);
                *rng_state = rng.next_u64() as i64;
                delay
            }
            DelayMode::Fixed => self.fixed_delay_values.customer_delay as i64,
        }
    }
}

// Implement MemoryData trait so DelayConfig can be stored in memory components
impl rsim::core::components::state::MemoryData for DelayConfig {}

impl Cycle for DelayConfig {
    type Output = i64;
    
    fn cycle(&mut self) -> Option<Self::Output> {
        // DelayConfig is static, return 0 as output
        Some(0)
    }
}

// Implement MemoryComponent trait for DelayConfig using macro
impl_memory_component!(DelayConfig, {
    input: input,
    output: output
});