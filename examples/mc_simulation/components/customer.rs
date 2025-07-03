use crate::core::components::{Component, PortType};
use crate::core::components::module::{ProcessorModule, PortSpec};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Customer component that consumes burgers with random timing delays
/// Connects to burger buffer memory port for consuming finished burgers
/// Stores internal state (timer, counters, RNG seed) in memory
#[derive(Debug)]
pub struct Customer {
    /// Minimum delay in cycles
    min_delay: u32,
    /// Maximum delay in cycles
    max_delay: u32,
    /// RNG seed for deterministic timing
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
            remaining_cycles: u32 = 0,
            total_consumed: u64 = 0,
            rng_state: u64 = 11111
        });
        
        // Configuration from component (not stored in memory)
        let min_delay = memory_read!(ctx, "customer_state", "min_delay", u32, 1);
        let max_delay = memory_read!(ctx, "customer_state", "max_delay", u32, 5);
        
        // Read burger buffer status
        let burger_available = if let Ok(Some(count)) = ctx.memory.read::<u64>("burger_buffer", "data_count") {
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
            memory_write!(ctx, "burger_buffer", "to_subtract", 1u64)?;
            total_consumed += 1;
            
            // Start new consumption cycle with random delay
            let mut rng = StdRng::seed_from_u64(rng_state);
            remaining_cycles = rng.gen_range(min_delay..=max_delay);
            rng_state = rng.next_u64(); // Update RNG state
        }
        // If no burger available, just wait
        
        // Write updated state back to memory using macro
        memory_state_write!(ctx, "customer_state", remaining_cycles, total_consumed, rng_state);
        
        Ok(())
    }
});