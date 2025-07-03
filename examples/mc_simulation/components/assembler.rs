use crate::core::components::{Component, PortType};
use crate::core::components::module::{ProcessorModule, PortSpec};
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


impl Component for Assembler {
    fn define_ports() -> Vec<(String, PortType)> {
        vec![
            ("bread_buffer".to_string(), PortType::Memory),
            ("meat_buffer".to_string(), PortType::Memory),
            ("burger_buffer".to_string(), PortType::Memory),
            ("assembler_state".to_string(), PortType::Memory),
        ]
    }
    
    fn into_module() -> ProcessorModule {
        let memory_ports = vec![
            PortSpec::memory("bread_buffer"),
            PortSpec::memory("meat_buffer"),
            PortSpec::memory("burger_buffer"),
            PortSpec::memory("assembler_state"),
        ];
        
        ProcessorModule::new(
            "Assembler", 
            vec![], // no input ports
            vec![], // no output ports
            memory_ports,
            |ctx, _outputs| {
                // Assembler logic: check ingredients, manage timer, assemble burgers
                
                // Read internal state from memory
                let mut remaining_cycles = ctx.memory.read::<u32>("assembler_state", "remaining_cycles").unwrap_or(Some(0)).unwrap_or(0);
                let mut total_assembled = ctx.memory.read::<u64>("assembler_state", "total_assembled").unwrap_or(Some(0)).unwrap_or(0);
                let mut rng_state = ctx.memory.read::<u64>("assembler_state", "rng_state").unwrap_or(Some(98765)).unwrap_or(98765);
                
                // Configuration from component (not stored in memory)
                let min_delay = ctx.memory.read::<u32>("assembler_state", "min_delay").unwrap_or(Some(1)).unwrap_or(1);
                let max_delay = ctx.memory.read::<u32>("assembler_state", "max_delay").unwrap_or(Some(3)).unwrap_or(3);
                
                // Read bread buffer status
                let bread_available = if let Ok(Some(count)) = ctx.memory.read::<u64>("bread_buffer", "data_count") {
                    count > 0
                } else {
                    false
                };
                
                // Read meat buffer status
                let meat_available = if let Ok(Some(count)) = ctx.memory.read::<u64>("meat_buffer", "data_count") {
                    count > 0
                } else {
                    false
                };
                
                // Read burger buffer status
                let burger_buffer_full = if let Ok(Some(count)) = ctx.memory.read::<u64>("burger_buffer", "data_count") {
                    let capacity = ctx.memory.read::<u64>("burger_buffer", "capacity").unwrap_or(Some(10)).unwrap_or(10);
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
                    // Request to consume one bread
                    ctx.memory.write("bread_buffer", "to_subtract", 1u64)?;
                    
                    // Request to consume one meat
                    ctx.memory.write("meat_buffer", "to_subtract", 1u64)?;
                    
                    // Request to add one burger to burger buffer
                    ctx.memory.write("burger_buffer", "to_add", 1u64)?;
                    
                    total_assembled += 1;
                    
                    // Start new assembly cycle with random delay
                    let mut rng = StdRng::seed_from_u64(rng_state);
                    remaining_cycles = rng.gen_range(min_delay..=max_delay);
                    rng_state = rng.next_u64(); // Update RNG state
                }
                // If ingredients not available or buffer full, just wait
                
                // Write updated state back to memory
                ctx.memory.write("assembler_state", "remaining_cycles", remaining_cycles)?;
                ctx.memory.write("assembler_state", "total_assembled", total_assembled)?;
                ctx.memory.write("assembler_state", "rng_state", rng_state)?;
                
                Ok(())
            }
        )
    }
}