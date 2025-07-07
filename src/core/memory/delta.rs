use crate::core::types::ComponentId;
use std::collections::HashMap;

/// Represents a single memory write operation that occurred during parallel execution
#[derive(Debug)]
pub struct MemoryWrite {
    /// The memory component that was written to
    pub memory_id: ComponentId,
    /// The address within the memory component
    pub address: String,
    /// The data that was written (as a boxed Any for type erasure)
    pub data: Box<dyn std::any::Any + Send>,
    /// The component that performed the write (for debugging)
    pub writer_id: ComponentId,
}

/// Collection of memory writes that occurred during parallel execution
/// This tracks all memory changes that need to be applied to the main memory system
#[derive(Debug, Default)]
pub struct MemoryDelta {
    /// Map of memory writes: (memory_id, address) -> MemoryWrite
    /// Using a HashMap ensures that multiple writes to the same address are handled correctly
    /// (last write wins, which matches sequential behavior)
    writes: HashMap<(ComponentId, String), MemoryWrite>,
}

impl MemoryDelta {
    /// Create a new empty memory delta
    pub fn new() -> Self {
        Self {
            writes: HashMap::new(),
        }
    }

    /// Record a memory write operation
    pub fn record_write<T: std::any::Any + Send>(
        &mut self,
        memory_id: ComponentId,
        address: String,
        data: T,
        writer_id: ComponentId,
    ) {
        let write = MemoryWrite {
            memory_id: memory_id.clone(),
            address: address.clone(),
            data: Box::new(data),
            writer_id,
        };
        
        self.writes.insert((memory_id, address), write);
    }

    /// Get all memory writes
    pub fn get_writes(&self) -> &HashMap<(ComponentId, String), MemoryWrite> {
        &self.writes
    }

    /// Check if there are any writes
    pub fn is_empty(&self) -> bool {
        self.writes.is_empty()
    }

    /// Get the number of writes
    pub fn len(&self) -> usize {
        self.writes.len()
    }

    /// Merge another delta into this one
    /// Later writes override earlier ones (last write wins)
    pub fn merge(&mut self, other: MemoryDelta) {
        self.writes.extend(other.writes);
    }

    /// Extract all writes and clear the delta
    pub fn take_writes(&mut self) -> HashMap<(ComponentId, String), MemoryWrite> {
        std::mem::take(&mut self.writes)
    }
}