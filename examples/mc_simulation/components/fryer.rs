use rsim::*;
use rand::{Rng, RngCore, SeedableRng};
use rand::rngs::StdRng;

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
        // Read internal state from memory using macros
        memory_state!(ctx, "fryer_state", {
            remaining_cycles: i64 = 0,
            total_produced: i64 = 0,
            rng_state: i64 = 12345
        });
        
        // Configuration values - could be stored in memory or use instance values
        let min_delay = 3i64;  // In real impl, could use self.min_delay
        let max_delay = 7i64;
        
        // Read buffer status from memory (previous cycle state)
        let buffer_full = if let Ok(Some(count)) = ctx.memory.read::<i64>("meat_buffer", "data_count") {
            if let Ok(Some(capacity)) = ctx.memory.read::<i64>("meat_buffer", "capacity") {
                count >= capacity
            } else {
                false
            }
        } else {
            false // If can't read buffer, assume not full
        };
        
        // Process timer logic
        if remaining_cycles > 0 {
            // Still processing, decrement timer
            remaining_cycles -= 1;
        } else if !buffer_full {
            // Timer expired and buffer not full, produce meat and start new timer
            memory_write!(ctx, "meat_buffer", "to_add", 1i64);
            total_produced += 1;
            
            // Start new production cycle with random delay
            let mut rng = StdRng::seed_from_u64(rng_state as u64);
            remaining_cycles = rng.gen_range(min_delay..=max_delay);
            rng_state = rng.next_u64() as i64; // Update RNG state
        }
        // If buffer is full, just wait (don't start new timer)
        
        // Write updated state back to memory using macro
        memory_state_write!(ctx, "fryer_state", remaining_cycles, total_produced, rng_state);
        
        Ok(())
    }
});