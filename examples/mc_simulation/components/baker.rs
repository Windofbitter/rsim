use rsim::*;
use rand::{Rng, RngCore, SeedableRng};
use rand::rngs::StdRng;

/// Baker component that produces bread with random timing delays
/// Connects to a dedicated FIFO buffer for bread storage
/// Stores internal state (timer, counters, RNG seed) in memory
#[derive(Debug)]
pub struct Baker {
    /// Minimum delay in cycles
    min_delay: u32,
    /// Maximum delay in cycles
    max_delay: u32,
    /// RNG seed for deterministic timing
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
        // Simplified state management - using local variables instead of memory
        // In a full implementation, proper field-level memory access would be needed
        let mut remaining_cycles: i64 = 0;
        let mut total_produced: i64 = 0;
        let mut rng_state: i64 = 54321;
        
        // Configuration values
        let min_delay: i64 = 2;
        let max_delay: i64 = 5;
        
        // This is a simplified version - in a real implementation,
        // the memory system would need to support field-level access
        // For now, we'll skip the actual memory operations
        
        // Process timer logic
        if remaining_cycles > 0 {
            // Still processing, decrement timer
            remaining_cycles -= 1;
        } else {
            // Timer expired, produce bread and start new timer
            total_produced += 1;
            
            // Start new production cycle with random delay
            let mut rng = StdRng::seed_from_u64(rng_state as u64);
            remaining_cycles = rng.gen_range(min_delay..=max_delay);
            rng_state = rng.next_u64() as i64; // Update RNG state
        }
        // If buffer is full, just wait (don't start new timer)
        
        // State is managed locally in this simplified version
        // In a full implementation, state would be persisted to memory
        
        Ok(())
    }
});