use rsim::*;
use rand::{Rng, RngCore, SeedableRng};
use rand::rngs::StdRng;

/// Baker component that produces bread with random timing delays
/// Connects to a dedicated FIFO buffer for bread storage
/// Stores internal state (timer, counters, RNG seed) in memory
#[derive(Debug)]
pub struct Baker {
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

impl Baker {
    /// Create a new Baker with specified timing parameters and seed
    pub fn new(min_delay: u32, max_delay: u32, seed: u64) -> Self {
        Self {
            min_delay,
            max_delay,
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
        
        // Configuration values - could be stored in memory or use instance values
        let min_delay = 2i64;
        let max_delay = 5i64;
        
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
            
            // Start new production cycle with random delay
            let mut rng = StdRng::seed_from_u64(state.rng_state as u64);
            state.remaining_cycles = rng.gen_range(min_delay..=max_delay);
            state.rng_state = rng.next_u64() as i64; // Update RNG state
        }
        // If buffer is full, wait (don't start new timer)
        
        // Write updated buffer state back
        memory_write!(ctx, "bread_buffer", "buffer", buffer_state);
        
        // Write updated state back to memory
        memory_write!(ctx, "baker_state", "state", state);
        
        Ok(())
    }
});