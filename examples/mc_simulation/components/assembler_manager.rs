use crate::core::components::{Component, PortType};
use crate::core::components::module::{ProcessorModule, PortSpec};

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

impl Component for AssemblerManager {
    fn define_ports() -> Vec<(String, PortType)> {
        let mut ports = Vec::new();
        
        // Memory ports to read from bread and meat managers
        ports.push(("bread_manager".to_string(), PortType::Memory));
        ports.push(("meat_manager".to_string(), PortType::Memory));
        
        // Memory ports to write to 10 assembler buffers
        for i in 1..=10 {
            ports.push((format!("assembler_buffer_{}", i), PortType::Memory));
        }
        
        ports
    }
    
    fn into_module() -> ProcessorModule {
        let mut memory_ports = Vec::new();
        
        // Memory ports for reading from managers
        memory_ports.push(PortSpec::memory("bread_manager"));
        memory_ports.push(PortSpec::memory("meat_manager"));
        
        // Memory ports for writing to assembler buffers
        for i in 1..=10 {
            memory_ports.push(PortSpec::memory(&format!("assembler_buffer_{}", i)));
        }
        
        ProcessorModule::new(
            "AssemblerManager", 
            vec![], // no input ports
            vec![], // no output ports
            memory_ports,
            |ctx, _outputs| {
                // AssemblerManager logic: check for both bread and meat availability,
                // then distribute ingredient pairs to assembler buffers with available space
                
                // Check bread availability from bread manager
                let bread_available = if let Ok(Some(count)) = ctx.memory.read::<u64>("bread_manager", "bread_count") {
                    count > 0
                } else {
                    false
                };
                
                // Check meat availability from meat manager
                let meat_available = if let Ok(Some(count)) = ctx.memory.read::<u64>("meat_manager", "meat_count") {
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
                        let bread_space = if let Ok(Some(bread_count)) = ctx.memory.read::<u64>(&buffer_name, "bread_count") {
                            let bread_capacity = ctx.memory.read::<u64>(&buffer_name, "bread_capacity").unwrap_or(Some(10)).unwrap_or(10);
                            bread_count < bread_capacity
                        } else {
                            true // If can't read, assume buffer has space
                        };
                        
                        let meat_space = if let Ok(Some(meat_count)) = ctx.memory.read::<u64>(&buffer_name, "meat_count") {
                            let meat_capacity = ctx.memory.read::<u64>(&buffer_name, "meat_capacity").unwrap_or(Some(10)).unwrap_or(10);
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
                    let bread_count = ctx.memory.read::<u64>("bread_manager", "bread_count").unwrap_or(Some(0)).unwrap_or(0);
                    let meat_count = ctx.memory.read::<u64>("meat_manager", "meat_count").unwrap_or(Some(0)).unwrap_or(0);
                    let max_pairs = std::cmp::min(
                        std::cmp::min(bread_count, meat_count),
                        available_assembler_buffers.len() as u64
                    );
                    
                    // Distribute ingredient pairs to available assembler buffers
                    for pair_idx in 0..(max_pairs as usize) {
                        let assembler_buffer_id = available_assembler_buffers[pair_idx];
                        let buffer_name = format!("assembler_buffer_{}", assembler_buffer_id);
                        
                        // Request to consume ingredients from managers
                        ctx.memory.write("bread_manager", "bread_to_subtract", 1u64)?;
                        ctx.memory.write("meat_manager", "meat_to_subtract", 1u64)?;
                        
                        // Request to add ingredient pair to assembler buffer
                        ctx.memory.write(&buffer_name, "bread_to_add", 1u64)?;
                        ctx.memory.write(&buffer_name, "meat_to_add", 1u64)?;
                    }
                }
                
                Ok(())
            }
        )
    }
}