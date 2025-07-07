use rsim::*;

/// FIFO Memory component that can handle individual field writes
/// This creates a memory component that stores FIFOData and provides 
/// proper structured access to FIFO operations
#[derive(Clone, Debug)]
pub struct FIFOMemory {
    /// Current number of items in the buffer
    pub data_count: i64,
    /// Number of items to add in this cycle
    pub to_add: i64,
    /// Number of items to subtract in this cycle
    pub to_subtract: i64,
    /// Maximum capacity of the buffer
    pub capacity: i64,
}

impl FIFOMemory {
    /// Create a new FIFO memory with specified capacity
    pub fn new(capacity: i64) -> Self {
        Self {
            data_count: 0,
            to_add: 0,
            to_subtract: 0,
            capacity,
        }
    }

    /// Check if the buffer is full
    pub fn is_full(&self) -> bool {
        self.data_count >= self.capacity
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data_count == 0
    }

    /// Get available space in the buffer
    pub fn available_space(&self) -> i64 {
        self.capacity.saturating_sub(self.data_count)
    }

    /// Update the buffer state based on pending operations
    pub fn update(&mut self) {
        
        // First subtract items (consume)
        self.data_count = self.data_count.saturating_sub(self.to_subtract);
        
        // Then add items (produce) up to capacity
        let can_add = std::cmp::min(self.to_add, self.available_space());
        self.data_count += can_add;
        
        
        // Reset operation counters
        self.to_add = 0;
        self.to_subtract = 0;
    }
}

// Implement MemoryData trait so FIFOMemory can be stored in memory components
impl rsim::core::components::state::MemoryData for FIFOMemory {}

impl Cycle for FIFOMemory {
    type Output = i64;
    
    fn cycle(&mut self) -> Option<Self::Output> {
        // Apply pending operations before returning current count
        self.update();
        Some(self.data_count)
    }
}

// Implement MemoryComponent trait for FIFOMemory using macro
impl_memory_component!(FIFOMemory, {
    input: input,
    output: output
});