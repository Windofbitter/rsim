use crate::core::components::state::MemoryData;
use crate::core::components::module::MemoryModuleTrait;
use std::collections::HashMap;
use std::any::Any;

/// FIFO (First In, First Out) memory component for McDonald's simulation
/// Represents a queue buffer with capacity constraints and operation tracking
#[derive(Clone, Debug)]
pub struct FIFOData {
    /// Current number of items in the buffer
    pub data_count: u64,
    /// Number of items to add in this cycle
    pub to_add: u64,
    /// Number of items to subtract in this cycle
    pub to_subtract: u64,
    /// Maximum capacity of the buffer
    pub capacity: u64,
}

impl FIFOData {
    /// Create a new FIFO with specified capacity
    pub fn new(capacity: u64) -> Self {
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
    pub fn available_space(&self) -> u64 {
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

    /// Request to add items to the buffer
    pub fn request_add(&mut self, count: u64) {
        self.to_add = self.to_add.saturating_add(count);
    }

    /// Request to subtract items from the buffer
    pub fn request_subtract(&mut self, count: u64) {
        self.to_subtract = self.to_subtract.saturating_add(count);
    }
}

// Implement MemoryData trait so FIFOData can be stored in memory components
impl MemoryData for FIFOData {}

/// FIFO Memory Module implementation
pub struct FIFOModule {
    /// Memory identifier
    pub memory_id: String,
    /// Current state (gets written to during cycle)
    current_state: HashMap<String, FIFOData>,
    /// Snapshot from previous cycle (gets read from during cycle)
    snapshot: HashMap<String, FIFOData>,
    /// Default capacity for new FIFO instances
    default_capacity: u64,
}

impl FIFOModule {
    /// Create a new FIFO module with specified default capacity
    pub fn new(memory_id: &str, default_capacity: u64) -> Self {
        Self {
            memory_id: memory_id.to_string(),
            current_state: HashMap::new(),
            snapshot: HashMap::new(),
            default_capacity,
        }
    }

    /// Initialize a FIFO at a specific address with custom capacity
    pub fn initialize_fifo(&mut self, address: &str, capacity: u64) {
        let fifo_data = FIFOData::new(capacity);
        self.current_state.insert(address.to_string(), fifo_data.clone());
        self.snapshot.insert(address.to_string(), fifo_data);
    }

    /// Get FIFO data from snapshot (read operation)
    pub fn get_fifo(&self, address: &str) -> Option<&FIFOData> {
        self.snapshot.get(address)
    }

    /// Update FIFO data in current state (write operation)
    pub fn update_fifo(&mut self, address: &str, fifo_data: FIFOData) {
        self.current_state.insert(address.to_string(), fifo_data);
    }

    /// Get mutable FIFO data from current state (for internal updates)
    pub fn get_fifo_mut(&mut self, address: &str) -> Option<&mut FIFOData> {
        self.current_state.get_mut(address)
    }
}

impl MemoryModuleTrait for FIFOModule {
    fn memory_id(&self) -> &str {
        &self.memory_id
    }

    fn read_any(&self, address: &str) -> Option<Box<dyn Any + Send>> {
        self.snapshot.get(address).map(|data| {
            let boxed: Box<dyn Any + Send> = Box::new(data.clone());
            boxed
        })
    }

    fn write_any(&mut self, address: &str, data: Box<dyn Any + Send>) -> bool {
        if let Ok(fifo_data) = data.downcast::<FIFOData>() {
            self.current_state.insert(address.to_string(), *fifo_data);
            true
        } else {
            false
        }
    }

    fn create_snapshot(&mut self) {
        // Update all FIFO states before creating snapshot
        for (address, fifo_data) in self.current_state.iter_mut() {
            fifo_data.update();
        }
        
        // Create snapshot
        self.snapshot = self.current_state.clone();
    }

    fn clone_module(&self) -> Box<dyn MemoryModuleTrait> {
        Box::new(FIFOModule {
            memory_id: self.memory_id.clone(),
            current_state: self.current_state.clone(),
            snapshot: self.snapshot.clone(),
            default_capacity: self.default_capacity,
        })
    }
}