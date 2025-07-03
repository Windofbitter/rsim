use rsim::core::components::{Component, PortType};
use rsim::core::components::module::{ProcessorModule, PortSpec};
use rsim::impl_component;

/// AssemblerManager component that coordinates ingredient distribution to assemblers
/// Reads from bread and meat manager buffers and distributes ingredient pairs
/// to assembler buffers with available space in round-robin fashion
/// Connects to 2 memory ports (read bread/meat) + 10 memory ports (write to assembler buffers)
#[derive(Debug)]
pub struct AssemblerManager {
    /// Total ingredient pairs distributed
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
    pub fn total_distributed(&self) -> u64 {
        self.total_distributed
    }
}

impl_component!(AssemblerManager, "AssemblerManager", {
    inputs: [],
    outputs: [],
    memory: [
        bread_manager, meat_manager,
        assembler_buffer_1, assembler_buffer_2, assembler_buffer_3, assembler_buffer_4, assembler_buffer_5,
        assembler_buffer_6, assembler_buffer_7, assembler_buffer_8, assembler_buffer_9, assembler_buffer_10
    ],
    react: |ctx, _outputs| {
        // Check bread availability from bread manager
        let bread_available = if let Ok(Some(count)) = ctx.memory.read::<i64>("bread_manager", "bread_count") {
            count > 0
        } else {
            false
        };
        
        // Check meat availability from meat manager
        let meat_available = if let Ok(Some(count)) = ctx.memory.read::<i64>("meat_manager", "meat_count") {
            count > 0
        } else {
            false
        };
        
        // Only proceed if both bread and meat are available
        if bread_available && meat_available {
            // Find all assembler buffers with available space for ingredient pairs
            let mut available_assembler_buffers = Vec::new();
            for i in 1..=10 {
                let buffer_name = format!("assembler_buffer_{}", i);
                
                // Check if buffer has space for both bread and meat
                let bread_space = if let Ok(Some(bread_count)) = ctx.memory.read::<i64>(&buffer_name, "bread_count") {
                    let bread_capacity = memory_read!(ctx, &buffer_name, "bread_capacity", i64, 10);
                    bread_count < bread_capacity
                } else {
                    true // If can't read, assume buffer has space
                };
                
                let meat_space = if let Ok(Some(meat_count)) = ctx.memory.read::<i64>(&buffer_name, "meat_count") {
                    let meat_capacity = memory_read!(ctx, &buffer_name, "meat_capacity", i64, 10);
                    meat_count < meat_capacity
                } else {
                    true // If can't read, assume buffer has space
                };
                
                // Only add to list if both bread and meat can be added
                if bread_space && meat_space {
                    available_assembler_buffers.push(i);
                }
            }
            
            // Get the maximum number of ingredient pairs we can create this cycle
            // Limited by available bread, meat, or assembler buffer space
            let bread_count = memory_read!(ctx, "bread_manager", "bread_count", i64, 0);
            let meat_count = memory_read!(ctx, "meat_manager", "meat_count", i64, 0);
            let max_pairs = std::cmp::min(
                std::cmp::min(bread_count, meat_count),
                available_assembler_buffers.len() as i64
            );
            
            // Distribute ingredient pairs to available assembler buffers
            for pair_idx in 0..(max_pairs as usize) {
                let assembler_buffer_id = available_assembler_buffers[pair_idx];
                let buffer_name = format!("assembler_buffer_{}", assembler_buffer_id);
                
                // Request to consume ingredients from managers
                memory_write!(ctx, "bread_manager", "bread_to_subtract", 1i64)?;
                memory_write!(ctx, "meat_manager", "meat_to_subtract", 1i64)?;
                
                // Request to add ingredient pair to assembler buffer
                memory_write!(ctx, &buffer_name, "bread_to_add", 1i64)?;
                memory_write!(ctx, &buffer_name, "meat_to_add", 1i64)?;
            }
        }
        
        Ok(())
    }
});