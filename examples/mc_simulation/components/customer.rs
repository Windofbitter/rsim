use rsim::*;
use rand::{Rng, RngCore, SeedableRng};
use rand::rngs::StdRng;

/// Customer component that consumes burgers with random timing delays
/// Connects to burger buffer memory port for consuming finished burgers
/// Stores internal state (timer, counters, RNG seed) in memory
#[derive(Debug)]
pub struct Customer {
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

impl Customer {
    /// Create a new Customer with specified timing parameters and seed
    pub fn new(min_delay: u32, max_delay: u32, seed: u64) -> Self {
        Self {
            min_delay,
            max_delay,
            rng_seed: seed,
        }
    }
}

impl_component!(Customer, "Customer", {
    inputs: [],
    outputs: [],
    memory: [burger_buffer, customer_state],
    react: |ctx, _outputs| {
        use crate::components::component_states::CustomerState;
        use crate::components::fifo_memory::FIFOMemory;
        
        // Read current state from memory (previous cycle)
        let mut state = if let Ok(Some(current_state)) = ctx.memory.read::<CustomerState>("customer_state", "state") {
            current_state
        } else {
            // Initialize with default state if no previous state exists
            CustomerState::new()
        };
        
        // Configuration values - could be stored in memory or use instance values
        let min_delay = 1i64;
        let max_delay = 5i64;
        
        // Read burger buffer status
        let mut burger_buffer = if let Ok(Some(buffer)) = ctx.memory.read::<FIFOMemory>("burger_buffer", "buffer") {
            buffer
        } else {
            FIFOMemory::new(50) // Default capacity
        };
        
        // Process timer logic
        if state.remaining_cycles > 0 {
            // Still consuming, decrement timer
            state.remaining_cycles -= 1;
        } else if burger_buffer.data_count > 0 {
            // Timer expired and burger available, consume burger and start new timer
            burger_buffer.to_subtract += 1;
            state.total_consumed += 1;
            // Start new consumption cycle with random delay
            let mut rng = StdRng::seed_from_u64(state.rng_state as u64);
            state.remaining_cycles = rng.gen_range(min_delay..=max_delay);
            state.rng_state = rng.next_u64() as i64; // Update RNG state
        } else {
            // Waiting for burger
        }
        // If no burger available, just wait
        
        // Write updated burger buffer back
        memory_write!(ctx, "burger_buffer", "buffer", burger_buffer);
        
        // Write updated state back to memory
        memory_write!(ctx, "customer_state", "state", state);
        
        Ok(())
    }
});