use crate::core::components::{Component, PortType};
use crate::core::components::module::{ProcessorModule, PortSpec};
use rand::{Rng, SeedableRng};
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
        // Read internal state from memory using macros
        memory_state!(ctx, "baker_state", {
            remaining_cycles: u32 = 0,
            total_produced: u64 = 0,
            rng_state: u64 = 54321
        });
        
        // Configuration from component (not stored in memory)
        let min_delay = memory_read!(ctx, "baker_state", "min_delay", u32, 2);
        let max_delay = memory_read!(ctx, "baker_state", "max_delay", u32, 5);
        
        // Read buffer status from memory (previous cycle state)
        let buffer_full = if let Ok(Some(count)) = ctx.memory.read::<u64>("bread_buffer", "data_count") {
            let capacity = memory_read!(ctx, "bread_buffer", "capacity", u64, 10);
            count >= capacity
        } else {
            false // If can't read buffer, assume not full
        };
        
        // Process timer logic
        if remaining_cycles > 0 {
            // Still processing, decrement timer
            remaining_cycles -= 1;
        } else if !buffer_full {
            // Timer expired and buffer not full, produce bread and start new timer
            memory_write!(ctx, "bread_buffer", "to_add", 1u64)?;
            total_produced += 1;
            
            // Start new production cycle with random delay
            let mut rng = StdRng::seed_from_u64(rng_state);
            remaining_cycles = rng.gen_range(min_delay..=max_delay);
            rng_state = rng.next_u64(); // Update RNG state
        }
        // If buffer is full, just wait (don't start new timer)
        
        // Write updated state back to memory using macro
        memory_state_write!(ctx, "baker_state", remaining_cycles, total_produced, rng_state);
        
        Ok(())
    }
});