use rsim::*;

/// Assembler component that combines ingredient pairs into burgers
/// Connects to three memory ports: ingredient buffer, burger buffer, and assembler state
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
    memory: [ingredient_buffer, burger_buffer, assembler_state],
    react: |ctx, _outputs| {
        use crate::components::component_states::AssemblerState;
        use crate::components::fifo_memory::FIFOMemory;
        use crate::simulation_builder::DelayMode;
        
        // Read current state from memory (previous cycle)
        let mut state = if let Ok(Some(current_state)) = ctx.memory.read::<AssemblerState>("assembler_state", "state") {
            current_state
        } else {
            // Initialize with default state if no previous state exists
            AssemblerState::new()
        };
        
        // Read ingredient buffer status (contains ingredient pairs)
        let mut ingredient_buffer = if let Ok(Some(buffer)) = ctx.memory.read::<FIFOMemory>("ingredient_buffer", "buffer") {
            buffer
        } else {
            FIFOMemory::new(5) // Default capacity
        };
        
        // Read burger buffer status
        let mut burger_buffer = if let Ok(Some(buffer)) = ctx.memory.read::<FIFOMemory>("burger_buffer", "buffer") {
            buffer
        } else {
            FIFOMemory::new(50) // Default capacity
        };
        
        
        // Process timer logic
        if state.remaining_cycles > 0 {
            // Still assembling, decrement timer
            state.remaining_cycles -= 1;
        } else if ingredient_buffer.data_count > 0 && !burger_buffer.is_full() {
            // Timer expired and can assemble, consume ingredient pair and produce burger
            ingredient_buffer.to_subtract += 1;
            burger_buffer.to_add += 1;
            
            state.total_assembled += 1;
            
            // Use delay configuration from state
            state.remaining_cycles = match state.delay_config.delay_mode {
                DelayMode::Random => {
                    use rand::{Rng, RngCore, SeedableRng};
                    use rand::rngs::StdRng;
                    let mut rng = StdRng::seed_from_u64(state.rng_state as u64);
                    let delay = rng.gen_range(state.delay_config.min_delay as i64..=state.delay_config.max_delay as i64);
                    state.rng_state = rng.next_u64() as i64;
                    delay
                }
                DelayMode::Fixed => state.delay_config.fixed_delay as i64,
            };
        } else {
            // Waiting for ingredients or burger buffer space
        }
        
        // Write updated buffer states back
        memory_write!(ctx, "ingredient_buffer", "buffer", ingredient_buffer);
        memory_write!(ctx, "burger_buffer", "buffer", burger_buffer);
        
        // Write updated state back to memory
        memory_write!(ctx, "assembler_state", "state", state);
        
        Ok(())
    }
});