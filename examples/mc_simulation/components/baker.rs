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


impl Component for Baker {
    fn define_ports() -> Vec<(String, PortType)> {
        vec![
            ("bread_buffer".to_string(), PortType::Memory),
            ("baker_state".to_string(), PortType::Memory),
        ]
    }
    
    fn into_module() -> ProcessorModule {
        let memory_ports = vec![
            PortSpec::memory("bread_buffer"),
            PortSpec::memory("baker_state"),
        ];
        
        ProcessorModule::new(
            "Baker", 
            vec![], // no input ports
            vec![], // no output ports
            memory_ports,
            |ctx, _outputs| {
                // Baker logic: check buffer status, manage timer, produce bread
                
                // Read internal state from memory
                let mut remaining_cycles = ctx.memory.read::<u32>("baker_state", "remaining_cycles").unwrap_or(Some(0)).unwrap_or(0);
                let mut total_produced = ctx.memory.read::<u64>("baker_state", "total_produced").unwrap_or(Some(0)).unwrap_or(0);
                let mut rng_state = ctx.memory.read::<u64>("baker_state", "rng_state").unwrap_or(Some(54321)).unwrap_or(54321);
                
                // Configuration from component (not stored in memory)
                let min_delay = ctx.memory.read::<u32>("baker_state", "min_delay").unwrap_or(Some(2)).unwrap_or(2);
                let max_delay = ctx.memory.read::<u32>("baker_state", "max_delay").unwrap_or(Some(5)).unwrap_or(5);
                
                // Read buffer status from memory (previous cycle state)
                let buffer_full = if let Ok(Some(count)) = ctx.memory.read::<u64>("bread_buffer", "data_count") {
                    let capacity = ctx.memory.read::<u64>("bread_buffer", "capacity").unwrap_or(Some(10)).unwrap_or(10);
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
                    // Write request to add one bread item
                    ctx.memory.write("bread_buffer", "to_add", 1u64)?;
                    total_produced += 1;
                    
                    // Start new production cycle with random delay
                    let mut rng = StdRng::seed_from_u64(rng_state);
                    remaining_cycles = rng.gen_range(min_delay..=max_delay);
                    rng_state = rng.next_u64(); // Update RNG state
                }
                // If buffer is full, just wait (don't start new timer)
                
                // Write updated state back to memory
                ctx.memory.write("baker_state", "remaining_cycles", remaining_cycles)?;
                ctx.memory.write("baker_state", "total_produced", total_produced)?;
                ctx.memory.write("baker_state", "rng_state", rng_state)?;
                
                Ok(())
            }
        )
    }
}