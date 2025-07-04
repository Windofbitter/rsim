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
        // Read internal state from memory using macros
        memory_state!(ctx, "baker_state", {
            remaining_cycles: i64 = 0,
            total_produced: i64 = 0,
            rng_state: i64 = 54321
        });
        
        // Read configuration from memory (or use instance values)
        // For this example, we'll use the instance's configuration
        let min_delay = 2i64;  // In real impl, could read from memory or use self.min_delay
        let max_delay = 5i64;
        
        // Read bread buffer status to check if full
        let buffer_full = if let Ok(Some(count)) = ctx.memory.read::<i64>("bread_buffer", "data_count") {
            if let Ok(Some(capacity)) = ctx.memory.read::<i64>("bread_buffer", "capacity") {
                count >= capacity
            } else {
                false
            }
        } else {
            false
        };
        
        // Process timer logic
        if remaining_cycles > 0 {
            // Still baking, decrement timer
            remaining_cycles -= 1;
        } else if !buffer_full {
            // Timer expired and buffer not full, produce bread
            memory_write!(ctx, "bread_buffer", "to_add", 1i64);
            total_produced += 1;
            
            // Start new production cycle with random delay
            let mut rng = StdRng::seed_from_u64(rng_state as u64);
            remaining_cycles = rng.gen_range(min_delay..=max_delay);
            rng_state = rng.next_u64() as i64; // Update RNG state
        }
        // If buffer is full, wait (don't start new timer)
        
        // Write updated state back to memory
        memory_state_write!(ctx, "baker_state", remaining_cycles, total_produced, rng_state);
        
        Ok(())
    }
});