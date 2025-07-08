use rsim::*;

/// Fryer component that produces meat with random timing delays
/// Connects to a dedicated FIFO buffer for meat storage
/// Stores internal state (timer, counters, RNG seed) in memory
#[derive(Debug)]
pub struct Fryer {
    /// Minimum delay in cycles
    #[allow(dead_code)]
    min_delay: u32,
    /// Maximum delay in cycles
    #[allow(dead_code)]
    max_delay: u32,
    /// RNG seed for deterministic timing
    #[allow(dead_code)]
    rng_seed: u64,
}

impl Fryer {
    /// Create a new Fryer with specified timing parameters and seed
    pub fn new(min_delay: u32, max_delay: u32, seed: u64) -> Self {
        Self {
            min_delay,
            max_delay,
            rng_seed: seed,
        }
    }
}


impl_component!(Fryer, "Fryer", {
    inputs: [],
    outputs: [],
    memory: [meat_buffer, fryer_state],
    react: |ctx, _outputs| {
        use crate::components::component_states::FryerState;
        use crate::components::fifo_memory::FIFOMemory;
        use crate::simulation_builder::DelayMode;
        
        // Read current state from memory (previous cycle)
        let mut state = if let Ok(Some(current_state)) = ctx.memory.read::<FryerState>("fryer_state", "state") {
            current_state
        } else {
            // Initialize with default state if no previous state exists
            FryerState::new()
        };
        
        // Read current meat buffer state
        let mut buffer_state = if let Ok(Some(current_buffer)) = ctx.memory.read::<FIFOMemory>("meat_buffer", "buffer") {
            current_buffer
        } else {
            // Initialize buffer if doesn't exist
            FIFOMemory::new(10) // Default capacity of 10
        };
        
        // Process timer logic
        if state.remaining_cycles > 0 {
            // Still processing, decrement timer
            state.remaining_cycles -= 1;
        } else if !buffer_state.is_full() {
            // Timer expired and buffer not full, request to produce meat
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
                DelayMode::Fixed => state.delay_config.fixed_delay as i64,
            };
        } else {
            // Buffer is full, wait
        }
        // If buffer is full, wait (don't start new timer)
        
        // Write updated buffer state back
        memory_write!(ctx, "meat_buffer", "buffer", buffer_state);
        
        // Write updated state back to memory
        memory_write!(ctx, "fryer_state", "state", state);
        
        Ok(())
    }
});