use rsim::*;

/// MeatManager component that collects meat from all individual meat buffers
/// and forwards to the AssemblerManager when available
/// Connects to 10 meat buffer memory ports (read) + 1 assembler manager memory port (write)
#[derive(Debug)]
pub struct MeatManager {
    /// Total meat collected from all buffers
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
        meat_inventory_out
    ],
    react: |ctx, _outputs| {
        use crate::components::fifo_memory::FIFOMemory;
        
        // Find all buffers with available meat
        let mut available_buffers = Vec::new();
        for i in 1..=10 {
            let buffer_name = format!("meat_buffer_{}", i);
            if let Ok(Some(buffer)) = ctx.memory.read::<FIFOMemory>(&buffer_name, "buffer") {
                if buffer.data_count > 0 {
                    available_buffers.push(i);
                }
            }
        }
        
        // Read current inventory buffer state
        let mut inventory_buffer = if let Ok(Some(current_buffer)) = ctx.memory.read::<FIFOMemory>("meat_inventory_out", "buffer") {
            current_buffer
        } else {
            FIFOMemory::new(100) // Large capacity for aggregated meat
        };
        
        // Transfer meat from available input buffers to inventory
        let mut transferred = 0;
        for buffer_idx in &available_buffers {
            let input_buffer_name = format!("meat_buffer_{}", buffer_idx);
            
            // Read input buffer state
            if let Ok(Some(mut input_buffer)) = ctx.memory.read::<FIFOMemory>(&input_buffer_name, "buffer") {
                if input_buffer.data_count > 0 && !inventory_buffer.is_full() {
                    // Transfer one meat from input to inventory
                    input_buffer.to_subtract += 1;
                    inventory_buffer.to_add += 1;
                    transferred += 1;
                    
                    // Write updated input buffer back
                    memory_write!(ctx, &input_buffer_name, "buffer", input_buffer);
                }
            }
        }
        
        // Write updated inventory buffer back
        memory_write!(ctx, "meat_inventory_out", "buffer", inventory_buffer);
        
        Ok(())
    }
});