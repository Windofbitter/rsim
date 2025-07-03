use crate::core::components::{Component, PortType};
use crate::core::components::module::{ProcessorModule, PortSpec};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Fryer component that produces meat with random timing delays
/// Connects to a dedicated FIFO buffer for meat storage
#[derive(Debug)]
pub struct Fryer {
    /// Internal timer state for processing delays
    remaining_cycles: u32,
    /// Minimum delay in cycles
    min_delay: u32,
    /// Maximum delay in cycles
    max_delay: u32,
    /// Random number generator for deterministic timing
    rng: StdRng,
    /// Total meat produced by this fryer
    total_produced: u64,
}

impl Fryer {
    /// Create a new Fryer with specified timing parameters and seed
    pub fn new(min_delay: u32, max_delay: u32, seed: u64) -> Self {
        Self {
            remaining_cycles: 0,
            min_delay,
            max_delay,
            rng: StdRng::seed_from_u64(seed),
            total_produced: 0,
        }
    }

    /// Check if the fryer is currently processing (has remaining delay cycles)
    pub fn is_processing(&self) -> bool {
        self.remaining_cycles > 0
    }

    /// Start a new meat production cycle with random delay
    fn start_production(&mut self) {
        self.remaining_cycles = self.rng.gen_range(self.min_delay..=self.max_delay);
    }

    /// Get total meat produced by this fryer
    pub fn total_produced(&self) -> u64 {
        self.total_produced
    }
}

impl Component for Fryer {
    fn define_ports() -> Vec<(String, PortType)> {
        vec![
            ("meat_buffer".to_string(), PortType::Memory),
        ]
    }
    
    fn into_module() -> ProcessorModule {
        let memory_ports = vec![
            PortSpec::memory("meat_buffer"),
        ];
        
        ProcessorModule::new(
            "Fryer", 
            vec![], // no input ports
            vec![], // no output ports
            memory_ports,
            |ctx, _outputs| {
                // Fryer logic: check buffer status, manage timer, produce meat
                
                // Read buffer status from memory (previous cycle state)
                let buffer_full = if let Ok(Some(count)) = ctx.memory.read::<u64>("meat_buffer", "data_count") {
                    let capacity = ctx.memory.read::<u64>("meat_buffer", "capacity").unwrap_or(Some(10)).unwrap_or(10);
                    count >= capacity
                } else {
                    false // If can't read buffer, assume not full
                };
                
                // If buffer is not full, request to add meat
                if !buffer_full {
                    // Write request to add one meat item
                    ctx.memory.write("meat_buffer", "to_add", 1u64)?;
                }
                
                Ok(())
            }
        )
    }
}