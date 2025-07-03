use crate::core::components::{Component, PortType};
use crate::core::components::module::{ProcessorModule, PortSpec};

/// BreadManager component that collects bread from all individual bread buffers
/// and forwards to the AssemblerManager when available
/// Connects to 10 bread buffer memory ports (read) + 1 assembler manager memory port (write)
#[derive(Debug)]
pub struct BreadManager {
    /// Total bread collected from all buffers
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
    pub fn total_collected(&self) -> u64 {
        self.total_collected
    }
}

impl Component for BreadManager {
    fn define_ports() -> Vec<(String, PortType)> {
        let mut ports = Vec::new();
        
        // Memory ports to read from 10 bread buffers
        for i in 1..=10 {
            ports.push((format!("bread_buffer_{}", i), PortType::Memory));
        }
        
        // Memory port to write to assembler manager
        ports.push(("assembler_manager".to_string(), PortType::Memory));
        
        ports
    }
    
    fn into_module() -> ProcessorModule {
        let mut memory_ports = Vec::new();
        
        // Memory ports for reading from bread buffers
        for i in 1..=10 {
            memory_ports.push(PortSpec::memory(&format!("bread_buffer_{}", i)));
        }
        
        // Memory port for writing to assembler manager
        memory_ports.push(PortSpec::memory("assembler_manager"));
        
        ProcessorModule::new(
            "BreadManager", 
            vec![], // no input ports
            vec![], // no output ports
            memory_ports,
            |ctx, _outputs| {
                // BreadManager logic: collect bread from available buffers in round-robin fashion
                // and distribute to assembler buffers with available space
                
                // Find all buffers with available bread
                let mut available_buffers = Vec::new();
                for i in 1..=10 {
                    let buffer_name = format!("bread_buffer_{}", i);
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
                    if let Ok(Some(count)) = ctx.memory.read::<u64>(&buffer_name, "bread_count") {
                        let capacity = ctx.memory.read::<u64>(&buffer_name, "bread_capacity").unwrap_or(Some(10)).unwrap_or(10);
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
                    ctx.memory.write(&input_buffer_name, "to_subtract", 1u64)?;
                    
                    // Request to add bread to assembler buffer
                    ctx.memory.write(&output_buffer_name, "bread_to_add", 1u64)?;
                }
                
                Ok(())
            }
        )
    }
}