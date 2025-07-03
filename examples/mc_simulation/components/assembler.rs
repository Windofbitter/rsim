use rsim::core::components::{Component, PortType};
use rsim::core::components::module::{ProcessorModule, PortSpec};
use rsim::impl_component;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Assembler component that combines bread and meat into burgers
/// Connects to three memory ports: bread buffer, meat buffer, and burger buffer
/// Stores internal state (timer, counters, RNG seed) in memory
#[derive(Debug)]
pub struct Assembler {
    /// Minimum delay in cycles
    min_delay: u32,
    /// Maximum delay in cycles
    max_delay: u32,
    /// RNG seed for deterministic timing
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
            remaining_cycles: u32 = 0,
            total_assembled: u64 = 0,
            rng_state: u64 = 98765
        });
        
        // Configuration from component (not stored in memory)
        let min_delay = memory_read!(ctx, "assembler_state", "min_delay", u32, 1);
        let max_delay = memory_read!(ctx, "assembler_state", "max_delay", u32, 3);
        
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
            let capacity = memory_read!(ctx, "burger_buffer", "capacity", i64, 10);
            count >= capacity
        } else {
            false
        };
        
        // Process timer logic
        if remaining_cycles > 0 {
            // Still assembling, decrement timer
            remaining_cycles -= 1;
        } else if bread_available && meat_available && !burger_buffer_full {
            // Timer expired and can assemble, consume ingredients and start new timer
            memory_write!(ctx, "bread_buffer", "to_subtract", 1i64)?;
            memory_write!(ctx, "meat_buffer", "to_subtract", 1i64)?;
            memory_write!(ctx, "burger_buffer", "to_add", 1i64)?;
            
            total_assembled += 1;
            
            // Start new assembly cycle with random delay
            let mut rng = StdRng::seed_from_u64(rng_state);
            remaining_cycles = rng.gen_range(min_delay..=max_delay);
            rng_state = rng.next_u64(); // Update RNG state
        }
        // If ingredients not available or buffer full, just wait
        
        // Write updated state back to memory using macro
        memory_state_write!(ctx, "assembler_state", remaining_cycles, total_assembled, rng_state);
        
        Ok(())
    }
});