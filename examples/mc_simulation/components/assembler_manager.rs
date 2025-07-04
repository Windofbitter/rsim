use rsim::*;

/// AssemblerManager component that coordinates ingredient distribution to assemblers
/// Reads from bread and meat manager buffers and distributes ingredient pairs
/// to assembler buffers with available space in round-robin fashion
/// Connects to 2 memory ports (read bread/meat) + 10 memory ports (write to assembler buffers)
#[derive(Debug)]
pub struct AssemblerManager {
    /// Total ingredient pairs distributed
    #[allow(dead_code)]
    total_distributed: u64,
}

impl AssemblerManager {
    /// Create a new AssemblerManager
    pub fn new() -> Self {
        Self {
            total_distributed: 0,
        }
    }

    /// Get total ingredient pairs distributed by this manager
    #[allow(dead_code)]
    pub fn total_distributed(&self) -> u64 {
        self.total_distributed
    }
}

impl_component!(AssemblerManager, "AssemblerManager", {
    inputs: [],
    outputs: [],
    memory: [
        bread_inventory, meat_inventory,
        assembler_buffer_1, assembler_buffer_2, assembler_buffer_3, assembler_buffer_4, assembler_buffer_5,
        assembler_buffer_6, assembler_buffer_7, assembler_buffer_8, assembler_buffer_9, assembler_buffer_10
    ],
    react: |ctx, _outputs| {
        use crate::components::fifo_memory::FIFOMemory;
        
        // Read bread inventory buffer state
        let mut bread_inventory = if let Ok(Some(buffer)) = ctx.memory.read::<FIFOMemory>("bread_inventory", "buffer") {
            buffer
        } else {
            FIFOMemory::new(100)
        };
        
        // Read meat inventory buffer state
        let mut meat_inventory = if let Ok(Some(buffer)) = ctx.memory.read::<FIFOMemory>("meat_inventory", "buffer") {
            buffer
        } else {
            FIFOMemory::new(100)
        };
        
        // Only proceed if both bread and meat are available
        if bread_inventory.data_count > 0 && meat_inventory.data_count > 0 {
            // Find all assembler buffers with available space for ingredient pairs
            let mut available_assembler_buffers = Vec::new();
            for i in 1..=10 {
                let buffer_name = format!("assembler_buffer_{}", i);
                
                // Check if buffer has space for ingredients
                if let Ok(Some(buffer)) = ctx.memory.read::<FIFOMemory>(&buffer_name, "buffer") {
                    if !buffer.is_full() {
                        available_assembler_buffers.push(i);
                    }
                } else {
                    // If can't read, assume buffer has space
                    available_assembler_buffers.push(i);
                }
            }
            
            // Calculate max pairs based on actual available resources
            let max_pairs = std::cmp::min(
                std::cmp::min(bread_inventory.data_count, meat_inventory.data_count),
                available_assembler_buffers.len() as i64
            );
            
            // Only proceed if we can actually create at least one pair
            if max_pairs > 0 {
                // Distribute ingredient pairs to available assembler buffers
                for pair_idx in 0..(max_pairs as usize) {
                    let assembler_buffer_id = available_assembler_buffers[pair_idx];
                    let buffer_name = format!("assembler_buffer_{}", assembler_buffer_id);
                    
                    // Read assembler buffer state
                    if let Ok(Some(mut assembler_buffer)) = ctx.memory.read::<FIFOMemory>(&buffer_name, "buffer") {
                        if !assembler_buffer.is_full() && bread_inventory.data_count > 0 && meat_inventory.data_count > 0 {
                            // Consume ingredients from inventory
                            bread_inventory.to_subtract += 1;
                            meat_inventory.to_subtract += 1;
                            
                            // Add ingredient pair to assembler buffer
                            assembler_buffer.to_add += 1;
                            
                            // Write updated assembler buffer back
                            memory_write!(ctx, &buffer_name, "buffer", assembler_buffer);
                        }
                    }
                }
            }
        }
        
        // Write updated inventory buffers back
        memory_write!(ctx, "bread_inventory", "buffer", bread_inventory);
        memory_write!(ctx, "meat_inventory", "buffer", meat_inventory);
        
        Ok(())
    }
});