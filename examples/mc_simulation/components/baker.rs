use crate::core::components::{Component, PortType};
use crate::core::components::module::{ProcessorModule, PortSpec};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Baker component that produces bread with random timing delays
/// Connects to a dedicated FIFO buffer for bread storage
#[derive(Debug)]
pub struct Baker {
    /// Internal timer state for processing delays
    remaining_cycles: u32,
    /// Minimum delay in cycles
    min_delay: u32,
    /// Maximum delay in cycles
    max_delay: u32,
    /// Random number generator for deterministic timing
    rng: StdRng,
    /// Total bread produced by this baker
    total_produced: u64,
}

impl Baker {
    /// Create a new Baker with specified timing parameters and seed
    pub fn new(min_delay: u32, max_delay: u32, seed: u64) -> Self {
        Self {
            remaining_cycles: 0,
            min_delay,
            max_delay,
            rng: StdRng::seed_from_u64(seed),
            total_produced: 0,
        }
    }

    /// Check if the baker is currently processing (has remaining delay cycles)
    pub fn is_processing(&self) -> bool {
        self.remaining_cycles > 0
    }

    /// Start a new bread production cycle with random delay
    fn start_production(&mut self) {
        self.remaining_cycles = self.rng.gen_range(self.min_delay..=self.max_delay);
    }

    /// Get total bread produced by this baker
    pub fn total_produced(&self) -> u64 {
        self.total_produced
    }
}

impl Component for Baker {
    fn define_ports() -> Vec<(String, PortType)> {
        vec![
            ("bread_buffer".to_string(), PortType::Memory),
        ]
    }
    
    fn into_module() -> ProcessorModule {
        let memory_ports = vec![
            PortSpec::memory("bread_buffer"),
        ];
        
        ProcessorModule::new(
            "Baker", 
            vec![], // no input ports
            vec![], // no output ports
            memory_ports,
            |ctx, _outputs| {
                // Baker logic: check buffer status, manage timer, produce bread
                
                // Read buffer status from memory (previous cycle state)
                let buffer_full = if let Ok(Some(count)) = ctx.memory.read::<u64>("bread_buffer", "data_count") {
                    let capacity = ctx.memory.read::<u64>("bread_buffer", "capacity").unwrap_or(Some(10)).unwrap_or(10);
                    count >= capacity
                } else {
                    false // If can't read buffer, assume not full
                };
                
                // Get baker state (this would need to be stored in component state)
                // For now, using simple logic without persistent state
                
                // If buffer is not full, request to add bread
                if !buffer_full {
                    // Write request to add one bread item
                    ctx.memory.write("bread_buffer", "to_add", 1u64)?;
                }
                
                Ok(())
            }
        )
    }
}