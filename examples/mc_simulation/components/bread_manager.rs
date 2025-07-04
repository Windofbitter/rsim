use rsim::*;

/// BreadManager component that collects bread from all individual bread buffers
/// and forwards to the AssemblerManager when available
/// Connects to 10 bread buffer memory ports (read) + 1 assembler manager memory port (write)
#[derive(Debug)]
pub struct BreadManager {
    /// Total bread collected from all buffers
    #[allow(dead_code)]
    total_collected: u64,
}

impl BreadManager {
    /// Create a new BreadManager
    pub fn new() -> Self {
        Self {
            total_collected: 0,
        }
    }

    /// Get total bread collected by this manager
    #[allow(dead_code)]
    pub fn total_collected(&self) -> u64 {
        self.total_collected
    }
}

impl_component!(BreadManager, "BreadManager", {
    inputs: [],
    outputs: [],
    memory: [
        bread_buffer_1, bread_buffer_2, bread_buffer_3, bread_buffer_4, bread_buffer_5,
        bread_buffer_6, bread_buffer_7, bread_buffer_8, bread_buffer_9, bread_buffer_10,
        bread_inventory_out
    ],
    react: |ctx, _outputs| {
        use crate::components::fifo_memory::FIFOMemory;
        
        // Find all buffers with available bread
        let mut available_buffers = Vec::new();
        for i in 1..=10 {
            let buffer_name = format!("bread_buffer_{}", i);
            if let Ok(Some(buffer)) = ctx.memory.read::<FIFOMemory>(&buffer_name, "buffer") {
                if buffer.data_count > 0 {
                    available_buffers.push(i);
                }
            }
        }
        
        // Read current inventory buffer state
        let mut inventory_buffer = if let Ok(Some(current_buffer)) = ctx.memory.read::<FIFOMemory>("bread_inventory_out", "buffer") {
            current_buffer
        } else {
            FIFOMemory::new(100) // Large capacity for aggregated bread
        };
        
        // Transfer bread from available input buffers to inventory
        for buffer_idx in available_buffers {
            let input_buffer_name = format!("bread_buffer_{}", buffer_idx);
            
            // Read input buffer state
            if let Ok(Some(mut input_buffer)) = ctx.memory.read::<FIFOMemory>(&input_buffer_name, "buffer") {
                if input_buffer.data_count > 0 && !inventory_buffer.is_full() {
                    // Transfer one bread from input to inventory
                    input_buffer.to_subtract += 1;
                    inventory_buffer.to_add += 1;
                    
                    // Write updated input buffer back
                    memory_write!(ctx, &input_buffer_name, "buffer", input_buffer);
                }
            }
        }
        
        // Write updated inventory buffer back
        memory_write!(ctx, "bread_inventory_out", "buffer", inventory_buffer);
        
        Ok(())
    }
});