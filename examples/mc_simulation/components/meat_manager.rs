use crate::core::components::{Component, PortType};
use crate::core::components::module::{ProcessorModule, PortSpec};

/// MeatManager component that collects meat from all individual meat buffers
/// and forwards to the AssemblerManager when available
/// Connects to 10 meat buffer memory ports (read) + 1 assembler manager memory port (write)
#[derive(Debug)]
pub struct MeatManager {
    /// Total meat collected from all buffers
    total_collected: u64,
}

impl MeatManager {
    /// Create a new MeatManager
    pub fn new() -> Self {
        Self {
            total_collected: 0,
        }
    }

    /// Get total meat collected by this manager
    pub fn total_collected(&self) -> u64 {
        self.total_collected
    }
}

impl_component!(MeatManager, "MeatManager", {
    inputs: [],
    outputs: [],
    memory: [
        meat_buffer_1, meat_buffer_2, meat_buffer_3, meat_buffer_4, meat_buffer_5,
        meat_buffer_6, meat_buffer_7, meat_buffer_8, meat_buffer_9, meat_buffer_10,
        assembler_manager
    ],
    react: |ctx, _outputs| {
        // Find all buffers with available meat
        let mut available_buffers = Vec::new();
        for i in 1..=10 {
            let buffer_name = format!("meat_buffer_{}", i);
            if let Ok(Some(count)) = ctx.memory.read::<u64>(&buffer_name, "data_count") {
                if count > 0 {
                    available_buffers.push(i);
                }
            }
        }
        
        // Find all assembler buffers with available space
        let mut available_assembler_buffers = Vec::new();
        for i in 1..=10 {
            let buffer_name = format!("assembler_buffer_{}", i);
            if let Ok(Some(count)) = ctx.memory.read::<u64>(&buffer_name, "meat_count") {
                let capacity = memory_read!(ctx, &buffer_name, "meat_capacity", u64, 10);
                if count < capacity {
                    available_assembler_buffers.push(i);
                }
            } else {
                // If can't read, assume buffer has space
                available_assembler_buffers.push(i);
            }
        }
        
        // Distribute meat from available input buffers to available assembler buffers
        // Each cycle, distribute one meat per available assembler buffer
        let pairs_to_transfer = std::cmp::min(available_buffers.len(), available_assembler_buffers.len());
        
        for pair_idx in 0..pairs_to_transfer {
            let input_buffer_id = available_buffers[pair_idx];
            let output_buffer_id = available_assembler_buffers[pair_idx];
            
            let input_buffer_name = format!("meat_buffer_{}", input_buffer_id);
            let output_buffer_name = format!("assembler_buffer_{}", output_buffer_id);
            
            // Request to consume meat from input buffer
            memory_write!(ctx, &input_buffer_name, "to_subtract", 1u64)?;
            
            // Request to add meat to assembler buffer
            memory_write!(ctx, &output_buffer_name, "meat_to_add", 1u64)?;
        }
        
        Ok(())
    }
});