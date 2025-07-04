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
        assembler_manager
    ],
    react: |ctx, _outputs| {
        // Find all buffers with available bread
        let mut available_buffers = Vec::new();
        for i in 1..=10 {
            let buffer_name = format!("bread_buffer_{}", i);
            if let Ok(Some(count)) = ctx.memory.read::<i64>(&buffer_name, "data_count") {
                if count > 0 {
                    available_buffers.push(i);
                }
            }
        }
        
        // Find all assembler buffers with available space
        let mut available_assembler_buffers = Vec::new();
        for i in 1..=10 {
            let buffer_name = format!("assembler_buffer_{}", i);
            if let Ok(Some(count)) = ctx.memory.read::<i64>(&buffer_name, "bread_count") {
                memory_read!(ctx, &buffer_name, "bread_capacity", capacity: i64 = 10);
                if count < capacity {
                    available_assembler_buffers.push(i);
                }
            } else {
                // If can't read, assume buffer has space
                available_assembler_buffers.push(i);
            }
        }
        
        // Distribute bread from available input buffers to available assembler buffers
        // Each cycle, distribute one bread per available assembler buffer
        let pairs_to_transfer = std::cmp::min(available_buffers.len(), available_assembler_buffers.len());
        
        for pair_idx in 0..pairs_to_transfer {
            let input_buffer_id = available_buffers[pair_idx];
            let output_buffer_id = available_assembler_buffers[pair_idx];
            
            let input_buffer_name = format!("bread_buffer_{}", input_buffer_id);
            let output_buffer_name = format!("assembler_buffer_{}", output_buffer_id);
            
            // Request to consume bread from input buffer
            memory_write!(ctx, &input_buffer_name, "to_subtract", 1i64);
            
            // Request to add bread to assembler buffer
            memory_write!(ctx, &output_buffer_name, "bread_to_add", 1i64);
        }
        
        Ok(())
    }
});