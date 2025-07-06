use crate::core::components::state::MemoryData;
use std::collections::HashMap;

/// Trait for memory modules that can store and retrieve typed data
pub trait MemoryModuleTrait: Send {
    /// Get the memory ID for this module
    fn memory_id(&self) -> &str;
    
    /// Read data from memory (type-erased)
    fn read_any(&self, address: &str) -> Option<Box<dyn std::any::Any + Send>>;
    
    /// Write data to memory (type-erased)
    fn write_any(&mut self, address: &str, data: Box<dyn std::any::Any + Send>) -> bool;
    
    /// Create a snapshot for next cycle
    fn create_snapshot(&mut self);
    
    /// Execute cycle method on stored data objects that implement Cycle
    fn cycle(&mut self) -> Result<(), String>;
    
    /// Get a clone of this memory module
    fn clone_module(&self) -> Box<dyn MemoryModuleTrait>;
}

/// Concrete memory module implementation for specific data types
pub struct MemoryModule<T: MemoryData> {
    /// Memory identifier
    pub memory_id: String,
    /// Current state (gets written to during cycle)
    current_state: HashMap<String, T>,
    /// Snapshot from previous cycle (gets read from during cycle)
    snapshot: HashMap<String, T>,
}

impl<T: MemoryData> MemoryModule<T> {
    /// Create a new memory module with validation
    pub fn new(memory_id: &str) -> Self {
        Self {
            memory_id: memory_id.to_string(),
            current_state: HashMap::new(),
            snapshot: HashMap::new(),
        }
    }

    /// Validate that this memory module has the correct architecture constraints
    /// Memory modules should have single input/output and state management
    pub fn validate_architecture(&self) -> Result<(), String> {
        // Note: Memory modules don't have traditional ports like processing components
        // Their "ports" are the memory connection points managed by the proxy system
        // This validation ensures the memory module is properly configured
        if self.memory_id.is_empty() {
            return Err("Memory module must have a valid memory_id".to_string());
        }
        Ok(())
    }

    /// Read from snapshot (previous cycle data)
    pub fn read(&self, address: &str) -> Option<T> {
        self.snapshot.get(address).cloned()
    }

    /// Write to current state (affects next cycle)
    pub fn write(&mut self, address: &str, data: T) -> bool {
        self.current_state.insert(address.to_string(), data);
        true
    }
}

impl<T: MemoryData + crate::core::components::traits::Cycle> MemoryModuleTrait for MemoryModule<T> {
    fn memory_id(&self) -> &str {
        &self.memory_id
    }

    fn read_any(&self, address: &str) -> Option<Box<dyn std::any::Any + Send>> {
        self.snapshot.get(address).map(|data| {
            let boxed: Box<dyn std::any::Any + Send> = Box::new(data.clone());
            boxed
        })
    }

    fn write_any(&mut self, address: &str, data: Box<dyn std::any::Any + Send>) -> bool {
        if let Ok(typed_data) = data.downcast::<T>() {
            self.current_state.insert(address.to_string(), *typed_data);
            true
        } else {
            // Log type mismatch error for debugging
            eprintln!("Type mismatch error in memory module '{}' at address '{}': expected type '{}', got different type", 
                     self.memory_id, address, std::any::type_name::<T>());
            false
        }
    }

    fn create_snapshot(&mut self) {
        self.snapshot = self.current_state.clone();
    }
    
    fn cycle(&mut self) -> Result<(), String> {
        // Call cycle() on all stored data objects
        for (_address, data) in &mut self.current_state {
            data.cycle();
        }
        Ok(())
    }

    fn clone_module(&self) -> Box<dyn MemoryModuleTrait> {
        Box::new(MemoryModule {
            memory_id: self.memory_id.clone(),
            current_state: self.current_state.clone(),
            snapshot: self.snapshot.clone(),
        })
    }
}