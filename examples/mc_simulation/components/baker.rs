use rsim::*;
use crate::simulation_builder::DelayMode;

/// Baker component that produces bread with configurable timing delays
/// Connects to a dedicated FIFO buffer for bread storage
/// Stores internal state (timer, counters, RNG seed) in memory
#[derive(Debug)]
pub struct Baker {
    /// Delay configuration for this baker instance
    delay_config: crate::components::component_states::BakerDelayConfig,
    /// RNG seed for deterministic timing
    rng_seed: u64,
}

impl Baker {
    /// Create a new Baker with specified timing parameters and seed
    pub fn new(min_delay: u32, max_delay: u32, seed: u64) -> Self {
        Self {
            delay_config: crate::components::component_states::BakerDelayConfig {
                delay_mode: DelayMode::Random,
                min_delay,
                max_delay,
                fixed_delay: (min_delay + max_delay) / 2, // Default to middle of range
            },
            rng_seed: seed,
        }
    }
    
    /// Create a new Baker with delay configuration
    pub fn with_delay_config(min_delay: u32, max_delay: u32, fixed_delay: u32, delay_mode: DelayMode, seed: u64) -> Self {
        Self {
            delay_config: crate::components::component_states::BakerDelayConfig {
                delay_mode,
                min_delay,
                max_delay,
                fixed_delay,
            },
            rng_seed: seed,
        }
    }
    
}

impl_component!(Baker, "Baker", {
    inputs: [],
    outputs: [],
    memory: [bread_buffer, baker_state],
    react: |ctx, _outputs| {
        use crate::components::component_states::BakerState;
        use crate::components::fifo_memory::FIFOMemory;
        
        // Read current state from memory (previous cycle)
        let mut state = if let Ok(Some(current_state)) = ctx.memory.read::<BakerState>("baker_state", "state") {
            current_state
        } else {
            // Initialize with default state if no previous state exists
            BakerState::new()
        };
        
        
        // Read current bread buffer state
        let mut buffer_state = if let Ok(Some(current_buffer)) = ctx.memory.read::<FIFOMemory>("bread_buffer", "buffer") {
            current_buffer
        } else {
            // Initialize buffer if doesn't exist
            FIFOMemory::new(10) // Default capacity of 10
        };
        
        // Process timer logic
        if state.remaining_cycles > 0 {
            // Still baking, decrement timer
            state.remaining_cycles -= 1;
        } else if !buffer_state.is_full() {
            // Timer expired and buffer not full, request to produce bread
            buffer_state.to_add += 1;
            state.total_produced += 1;
            
            // Use delay configuration from state
            state.remaining_cycles = match state.delay_config.delay_mode {
                DelayMode::Random => {
                    use rand::{Rng, RngCore, SeedableRng};
                    use rand::rngs::StdRng;
                    let mut rng = StdRng::seed_from_u64(state.rng_state as u64);
                    let delay = rng.gen_range(state.delay_config.min_delay as i64..=state.delay_config.max_delay as i64);
                    state.rng_state = rng.next_u64() as i64;
                    delay
                }
                DelayMode::Fixed => {
                    state.delay_config.fixed_delay as i64
                },
            };
        } else {
            // Buffer is full, wait
        }
        // If buffer is full, wait (don't start new timer)
        
        // Write updated buffer state back
        memory_write!(ctx, "bread_buffer", "buffer", buffer_state);
        
        // Write updated state back to memory
        memory_write!(ctx, "baker_state", "state", state);
        
        Ok(())
    }
});