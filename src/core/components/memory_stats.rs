/// Memory statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Total number of memory addresses
    pub total_addresses: usize,
    /// Number of active (non-empty) addresses
    pub active_addresses: usize,
    /// Memory usage in bytes (estimated)
    pub memory_usage_bytes: usize,
    /// Number of read operations in current cycle
    pub read_count: usize,
    /// Number of write operations in current cycle
    pub write_count: usize,
}

impl MemoryStats {
    /// Create new empty memory stats
    pub fn new() -> Self {
        Self {
            total_addresses: 0,
            active_addresses: 0,
            memory_usage_bytes: 0,
            read_count: 0,
            write_count: 0,
        }
    }
    
    /// Reset cycle-specific counters
    pub fn reset_cycle_counters(&mut self) {
        self.read_count = 0;
        self.write_count = 0;
    }
    
    /// Update memory usage statistics
    pub fn update_usage(&mut self, total: usize, active: usize, bytes: usize) {
        self.total_addresses = total;
        self.active_addresses = active;
        self.memory_usage_bytes = bytes;
    }
    
    /// Increment read counter
    pub fn increment_reads(&mut self) {
        self.read_count += 1;
    }
    
    /// Increment write counter
    pub fn increment_writes(&mut self) {
        self.write_count += 1;
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self::new()
    }
}