use rsim::*;
use rand::{Rng, RngCore, SeedableRng};
use rand::rngs::StdRng;

/// Assembler component that combines bread and meat into burgers
/// Connects to three memory ports: bread buffer, meat buffer, and burger buffer
/// Stores internal state (timer, counters, RNG seed) in memory
#[derive(Debug)]
pub struct Assembler {
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

impl Assembler {
    /// Create a new Assembler with specified timing parameters and seed
    pub fn new(min_delay: u32, max_delay: u32, seed: u64) -> Self {
        Self {
            min_delay,
            max_delay,
            rng_seed: seed,
        }
    }
}


impl_component!(Assembler, "Assembler", {
    inputs: [],
    outputs: [],
    memory: [bread_buffer, meat_buffer, burger_buffer, assembler_state],
    react: |ctx, _outputs| {
        // Read internal state from memory using macros
        memory_state!(ctx, "assembler_state", {
            remaining_cycles: i64 = 0,
            total_assembled: i64 = 0,
            rng_state: i64 = 98765
        });
        
        // Configuration values - could be stored in memory or use instance values
        let min_delay = 1i64;  // In real impl, could use self.min_delay
        let max_delay = 3i64;
        
        // Read bread buffer status
        let bread_available = if let Ok(Some(count)) = ctx.memory.read::<i64>("bread_buffer", "data_count") {
            count > 0
        } else {
            false
        };
        
        // Read meat buffer status
        let meat_available = if let Ok(Some(count)) = ctx.memory.read::<i64>("meat_buffer", "data_count") {
            count > 0
        } else {
            false
        };
        
        // Read burger buffer status
        let burger_buffer_full = if let Ok(Some(count)) = ctx.memory.read::<i64>("burger_buffer", "data_count") {
            if let Ok(Some(capacity)) = ctx.memory.read::<i64>("burger_buffer", "capacity") {
                count >= capacity
            } else {
                false
            }
        } else {
            false
        };
        
        // Process timer logic
        if remaining_cycles > 0 {
            // Still assembling, decrement timer
            remaining_cycles -= 1;
        } else if bread_available && meat_available && !burger_buffer_full {
            // Timer expired and can assemble, consume ingredients and start new timer
            memory_write!(ctx, "bread_buffer", "to_subtract", 1i64);
            memory_write!(ctx, "meat_buffer", "to_subtract", 1i64);
            memory_write!(ctx, "burger_buffer", "to_add", 1i64);
            
            total_assembled += 1;
            
            // Start new assembly cycle with random delay
            let mut rng = StdRng::seed_from_u64(rng_state as u64);
            remaining_cycles = rng.gen_range(min_delay..=max_delay);
            rng_state = rng.next_u64() as i64; // Update RNG state
        }
        // If ingredients not available or buffer full, just wait
        
        // Write updated state back to memory using macro
        memory_state_write!(ctx, "assembler_state", remaining_cycles, total_assembled, rng_state);
        
        Ok(())
    }
});