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
        // Read internal state from memory using macros
        memory_state!(ctx, "customer_state", {
            remaining_cycles: i64 = 0,
            total_consumed: i64 = 0,
            rng_state: i64 = 11111
        });
        
        // Configuration values - could be stored in memory or use instance values
        let min_delay = 1i64;  // In real impl, could use self.min_delay
        let max_delay = 5i64;
        
        // Read burger buffer status
        let burger_available = if let Ok(Some(count)) = ctx.memory.read::<i64>("burger_buffer", "data_count") {
            count > 0
        } else {
            false
        };
        
        // Process timer logic
        if remaining_cycles > 0 {
            // Still consuming, decrement timer
            remaining_cycles -= 1;
        } else if burger_available {
            // Timer expired and burger available, consume burger and start new timer
            memory_write!(ctx, "burger_buffer", "to_subtract", 1i64);
            total_consumed += 1;
            
            // Start new consumption cycle with random delay
            let mut rng = StdRng::seed_from_u64(rng_state as u64);
            remaining_cycles = rng.gen_range(min_delay..=max_delay);
            rng_state = rng.next_u64() as i64; // Update RNG state
        }
        // If no burger available, just wait
        
        // Write updated state back to memory using macro
        memory_state_write!(ctx, "customer_state", remaining_cycles, total_consumed, rng_state);
        
        Ok(())
    }
});