use crate::core::components::{Component, PortType};
use crate::core::components::module::{ProcessorModule, PortSpec};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Assembler component that combines bread and meat into burgers
/// Connects to three memory ports: bread buffer, meat buffer, and burger buffer
#[derive(Debug)]
pub struct Assembler {
    /// Internal timer state for processing delays
    remaining_cycles: u32,
    /// Minimum delay in cycles
    min_delay: u32,
    /// Maximum delay in cycles
    max_delay: u32,
    /// Random number generator for deterministic timing
    rng: StdRng,
    /// Total burgers assembled
    total_assembled: u64,
}

impl Assembler {
    /// Create a new Assembler with specified timing parameters and seed
    pub fn new(min_delay: u32, max_delay: u32, seed: u64) -> Self {
        Self {
            remaining_cycles: 0,
            min_delay,
            max_delay,
            rng: StdRng::seed_from_u64(seed),
            total_assembled: 0,
        }
    }

    /// Check if the assembler is currently processing (has remaining delay cycles)
    pub fn is_processing(&self) -> bool {
        self.remaining_cycles > 0
    }

    /// Start a new burger assembly cycle with random delay
    fn start_assembly(&mut self) {
        self.remaining_cycles = self.rng.gen_range(self.min_delay..=self.max_delay);
    }

    /// Get total burgers assembled
    pub fn total_assembled(&self) -> u64 {
        self.total_assembled
    }
}

impl Component for Assembler {
    fn define_ports() -> Vec<(String, PortType)> {
        vec![
            ("bread_buffer".to_string(), PortType::Memory),
            ("meat_buffer".to_string(), PortType::Memory),
            ("burger_buffer".to_string(), PortType::Memory),
        ]
    }
    
    fn into_module() -> ProcessorModule {
        let memory_ports = vec![
            PortSpec::memory("bread_buffer"),
            PortSpec::memory("meat_buffer"),
            PortSpec::memory("burger_buffer"),
        ];
        
        ProcessorModule::new(
            "Assembler", 
            vec![], // no input ports
            vec![], // no output ports
            memory_ports,
            |ctx, _outputs| {
                // Assembler logic: check bread availability, meat availability, and burger buffer capacity
                
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
                
                // Only process if both bread and meat are available and burger buffer is not full
                if bread_available && meat_available && !burger_buffer_full {
                    // Request to consume one bread
                    ctx.memory.write("bread_buffer", "to_subtract", 1u64)?;
                    
                    // Request to consume one meat
                    ctx.memory.write("meat_buffer", "to_subtract", 1u64)?;
                    
                    // Request to add one burger to burger buffer
                    ctx.memory.write("burger_buffer", "to_add", 1u64)?;
                }
                
                Ok(())
            }
        )
    }
}