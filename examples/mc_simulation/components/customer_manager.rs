use rsim::*;

/// CustomerManager component that collects burgers from all assembler outputs
/// and distributes them to customer buffers with available space
/// Connects to 10 assembler output memory ports (read) + 10 customer buffer memory ports (write)
#[derive(Debug)]
pub struct CustomerManager {
    /// Total burgers distributed
    #[allow(dead_code)]
    total_distributed: u64,
}

impl CustomerManager {
    /// Create a new CustomerManager
    pub fn new() -> Self {
        Self {
            total_distributed: 0,
        }
    }

    /// Get total burgers distributed by this manager
    #[allow(dead_code)]
    pub fn total_distributed(&self) -> u64 {
        self.total_distributed
    }
}

impl_component!(CustomerManager, "CustomerManager", {
    inputs: [],
    outputs: [],
    memory: [
        assembler_output_1, assembler_output_2, assembler_output_3, assembler_output_4, assembler_output_5,
        assembler_output_6, assembler_output_7, assembler_output_8, assembler_output_9, assembler_output_10,
        customer_buffer_1, customer_buffer_2, customer_buffer_3, customer_buffer_4, customer_buffer_5,
        customer_buffer_6, customer_buffer_7, customer_buffer_8, customer_buffer_9, customer_buffer_10
    ],
    react: |ctx, _outputs| {
        use crate::components::fifo_memory::FIFOMemory;
        
        // Find all assembler outputs with available burgers
        let mut available_burger_sources = Vec::new();
        for i in 1..=10 {
            let source_name = format!("assembler_output_{}", i);
            if let Ok(Some(buffer)) = ctx.memory.read::<FIFOMemory>(&source_name, "buffer") {
                if buffer.data_count > 0 {
                    available_burger_sources.push(i);
                }
            }
        }
        
        // Find all customer buffers with available space
        let mut available_customer_buffers = Vec::new();
        for i in 1..=10 {
            let buffer_name = format!("customer_buffer_{}", i);
            if let Ok(Some(buffer)) = ctx.memory.read::<FIFOMemory>(&buffer_name, "buffer") {
                if !buffer.is_full() {
                    available_customer_buffers.push(i);
                }
            } else {
                // If can't read, assume buffer has space
                available_customer_buffers.push(i);
            }
        }
        
        // Distribute burgers from available assembler outputs to available customer buffers
        // Each cycle, distribute one burger per available customer buffer
        let burgers_to_transfer = std::cmp::min(available_burger_sources.len(), available_customer_buffers.len());
        
        for transfer_idx in 0..burgers_to_transfer {
            let source_id = available_burger_sources[transfer_idx];
            let customer_buffer_id = available_customer_buffers[transfer_idx];
            
            let source_name = format!("assembler_output_{}", source_id);
            let customer_buffer_name = format!("customer_buffer_{}", customer_buffer_id);
            
            // Read and update assembler output buffer
            if let Ok(Some(mut source_buffer)) = ctx.memory.read::<FIFOMemory>(&source_name, "buffer") {
                if source_buffer.data_count > 0 {
                    // Read and update customer buffer
                    if let Ok(Some(mut customer_buffer)) = ctx.memory.read::<FIFOMemory>(&customer_buffer_name, "buffer") {
                        if !customer_buffer.is_full() {
                            // Transfer burger from assembler output to customer buffer
                            source_buffer.to_subtract += 1;
                            customer_buffer.to_add += 1;
                            
                            // Write updated buffers back
                            memory_write!(ctx, &source_name, "buffer", source_buffer);
                            memory_write!(ctx, &customer_buffer_name, "buffer", customer_buffer);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
});