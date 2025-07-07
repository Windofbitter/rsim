use crate::core::components::state::MemoryData;
use crate::core::components::module::MemoryModuleTrait;
use crate::core::types::ComponentId;
use crate::core::memory::delta::{MemoryDelta, MemoryWrite};
use std::collections::HashMap;
use std::sync::mpsc;

/// Simplified memory proxy for the new direct API
/// 
/// This proxy provides access to memory components for processing components
/// that need to read/write memory during their evaluation.
pub struct MemoryProxy<'a> {
    /// Memory connections mapping: (component_id, port) -> memory_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,
    /// Current component ID for context
    component_id: ComponentId,
    /// Registry of actual memory modules (integrated with snapshot system)
    memory_modules: Option<&'a mut HashMap<ComponentId, Box<dyn MemoryModuleTrait>>>,
    /// Owned memory modules for parallel execution
    owned_memory_modules: Option<HashMap<ComponentId, Box<dyn MemoryModuleTrait>>>,
    /// Subset of memory component IDs for this specific component (parallel execution)
    memory_components_subset: Option<Vec<ComponentId>>,
    /// Memory delta tracking for parallel execution
    memory_delta: Option<MemoryDelta>,
    /// Channel sender for memory writes (for channel-based memory synchronization)
    memory_write_sender: Option<mpsc::Sender<MemoryWrite>>,
}

impl<'a> MemoryProxy<'a> {
    /// Create a new memory proxy
    pub fn new(
        memory_connections: HashMap<(ComponentId, String), ComponentId>,
        component_id: ComponentId,
        memory_modules: &'a mut HashMap<ComponentId, Box<dyn MemoryModuleTrait>>,
    ) -> Self {
        Self {
            memory_connections,
            component_id,
            memory_modules: Some(memory_modules),
            owned_memory_modules: None,
            memory_components_subset: None,
            memory_delta: None,
            memory_write_sender: None,
        }
    }

    /// Create a memory proxy with a subset of memory components for parallel execution
    /// This eliminates HashMap contention by giving each component only the memory it needs
    pub fn new_with_component_subset(
        memory_connections: HashMap<(ComponentId, String), ComponentId>,
        component_id: ComponentId,
        memory_modules: &'a mut HashMap<ComponentId, Box<dyn MemoryModuleTrait>>,
        memory_subset: &[ComponentId],
    ) -> Self {
        Self {
            memory_connections,
            component_id,
            memory_modules: Some(memory_modules),
            owned_memory_modules: None,
            memory_components_subset: Some(memory_subset.to_vec()),
            memory_delta: None,
            memory_write_sender: None,
        }
    }
    
    /// Create a memory proxy with owned memory components for parallel execution
    /// This version takes ownership of memory components to avoid borrowing issues
    pub fn new_with_owned_components(
        memory_connections: HashMap<(ComponentId, String), ComponentId>,
        component_id: ComponentId,
        owned_memory_modules: HashMap<ComponentId, Box<dyn MemoryModuleTrait>>,
        memory_subset: &[ComponentId],
    ) -> Self {
        Self {
            memory_connections,
            component_id,
            memory_modules: None,
            owned_memory_modules: Some(owned_memory_modules),
            memory_components_subset: Some(memory_subset.to_vec()),
            memory_delta: None,
            memory_write_sender: None,
        }
    }

    /// Create a memory proxy with delta tracking for parallel execution
    /// This version tracks all memory writes for later application to the main memory system
    pub fn new_with_delta_tracking(
        memory_connections: HashMap<(ComponentId, String), ComponentId>,
        component_id: ComponentId,
        owned_memory_modules: HashMap<ComponentId, Box<dyn MemoryModuleTrait>>,
        memory_subset: &[ComponentId],
    ) -> Self {
        Self {
            memory_connections,
            component_id,
            memory_modules: None,
            owned_memory_modules: Some(owned_memory_modules),
            memory_components_subset: Some(memory_subset.to_vec()),
            memory_delta: Some(MemoryDelta::new()),
            memory_write_sender: None,
        }
    }

    /// Create a memory proxy with channel-based memory synchronization for parallel execution
    /// This version sends memory writes through channels instead of writing directly
    pub fn new_with_channel_synchronization(
        memory_connections: HashMap<(ComponentId, String), ComponentId>,
        component_id: ComponentId,
        owned_memory_modules: HashMap<ComponentId, Box<dyn MemoryModuleTrait>>,
        memory_subset: &[ComponentId],
        memory_write_sender: mpsc::Sender<MemoryWrite>,
    ) -> Self {
        Self {
            memory_connections,
            component_id,
            memory_modules: None,
            owned_memory_modules: Some(owned_memory_modules),
            memory_components_subset: Some(memory_subset.to_vec()),
            memory_delta: None,
            memory_write_sender: Some(memory_write_sender),
        }
    }

    /// Read typed data from memory (reads from snapshot - previous cycle data)
    pub fn read<T: MemoryData>(&self, port: &str, address: &str) -> Result<Option<T>, String> {
        let mem_id = self
            .memory_connections
            .get(&(self.component_id.clone(), port.to_string()))
            .ok_or_else(|| format!("Memory port '{}' not connected for component '{}'", port, self.component_id))?;

        // Check if we have a subset and if this memory component is allowed
        if let Some(ref subset) = self.memory_components_subset {
            if !subset.contains(mem_id) {
                return Err(format!("Memory component '{}' not in subset for component '{}'", mem_id, self.component_id));
            }
        }

        let memory_module = if let Some(ref modules) = self.memory_modules {
            modules.get(mem_id)
        } else if let Some(ref modules) = self.owned_memory_modules {
            modules.get(mem_id)
        } else {
            return Err("No memory modules available in proxy".to_string());
        };
        
        if let Some(memory_module) = memory_module {
            if let Some(data_box) = memory_module.read_any(address) {
                // Try to downcast to the requested type
                if let Ok(typed_data) = data_box.downcast::<T>() {
                    Ok(Some(*typed_data))
                } else {
                    Err(format!("Type mismatch reading from memory address '{}' in memory '{}'", address, mem_id))
                }
            } else {
                Ok(None)
            }
        } else {
            Err(format!("Memory module '{}' not found in proxy registry", mem_id))
        }
    }

    /// Write typed data to memory (writes to current_state - affects next cycle)
    pub fn write<T: MemoryData>(&mut self, port: &str, address: &str, data: T) -> Result<(), String> {
        let mem_id = self
            .memory_connections
            .get(&(self.component_id.clone(), port.to_string()))
            .ok_or_else(|| format!("Memory port '{}' not connected for component '{}'", port, self.component_id))?;

        // Check if we have a subset and if this memory component is allowed
        if let Some(ref subset) = self.memory_components_subset {
            if !subset.contains(mem_id) {
                return Err(format!("Memory component '{}' not in subset for component '{}'", mem_id, self.component_id));
            }
        }

        // If channel-based synchronization is enabled, send the write through the channel
        if let Some(ref sender) = self.memory_write_sender {
            let memory_write = MemoryWrite {
                memory_id: mem_id.clone(),
                address: address.to_string(),
                data: Box::new(data),
                writer_id: self.component_id.clone(),
            };
            
            sender.send(memory_write)
                .map_err(|e| format!("Failed to send memory write through channel: {}", e))?;
            
            return Ok(());
        }

        // Clone data for delta tracking before consuming it
        let data_for_delta = data.clone();

        let memory_module = if let Some(ref mut modules) = self.memory_modules {
            modules.get_mut(mem_id)
        } else if let Some(ref mut modules) = self.owned_memory_modules {
            modules.get_mut(mem_id)
        } else {
            return Err("No memory modules available in proxy".to_string());
        };
        
        if let Some(memory_module) = memory_module {
            let data_box: Box<dyn std::any::Any + Send> = Box::new(data);
            if memory_module.write_any(address, data_box) {
                // If delta tracking is enabled, record the write
                if let Some(ref mut delta) = self.memory_delta {
                    delta.record_write(
                        mem_id.clone(),
                        address.to_string(),
                        data_for_delta,
                        self.component_id.clone(),
                    );
                }
                Ok(())
            } else {
                Err(format!("Failed to write to memory address '{}' in memory '{}' - type mismatch", address, mem_id))
            }
        } else {
            Err(format!("Memory module '{}' not found in proxy registry", mem_id))
        }
    }

    /// Check if a memory port is connected
    pub fn is_connected(&self, port: &str) -> bool {
        self.memory_connections.contains_key(&(self.component_id.clone(), port.to_string()))
    }

    /// Get connected memory component ID for a port
    pub fn get_memory_id(&self, port: &str) -> Option<&ComponentId> {
        self.memory_connections.get(&(self.component_id.clone(), port.to_string()))
    }

    /// Add a memory module to the proxy registry
    pub fn register_memory_module(&mut self, memory_id: ComponentId, module: Box<dyn MemoryModuleTrait>) {
        if let Some(ref mut modules) = self.memory_modules {
            modules.insert(memory_id, module);
        } else if let Some(ref mut modules) = self.owned_memory_modules {
            modules.insert(memory_id, module);
        }
    }

    /// Check if a memory module is registered
    pub fn has_memory_module(&self, memory_id: &ComponentId) -> bool {
        if let Some(ref modules) = self.memory_modules {
            modules.contains_key(memory_id)
        } else if let Some(ref modules) = self.owned_memory_modules {
            modules.contains_key(memory_id)
        } else {
            false
        }
    }
    
    /// Extract updated memory components for merging back to main memory
    /// This method is used in parallel execution to return modified memory components
    pub fn take_updated_memory_components(&mut self) -> HashMap<ComponentId, Box<dyn MemoryModuleTrait>> {
        std::mem::replace(&mut self.owned_memory_modules, None).unwrap_or_default()
    }

    /// Extract memory delta for parallel execution
    /// This method is used to get the memory changes that need to be applied to the main memory system
    pub fn take_memory_delta(&mut self) -> Option<MemoryDelta> {
        self.memory_delta.take()
    }

    /// Extract updated memory components by consuming the proxy
    /// This method is used in parallel execution to avoid borrowing issues
    pub fn into_updated_memory_components(self) -> HashMap<ComponentId, Box<dyn MemoryModuleTrait>> {
        self.owned_memory_modules.unwrap_or_default()
    }

    /// Get a reference to the owned memory modules
    /// This method allows access to the memory modules without consuming the proxy
    pub fn get_owned_memory_modules(&self) -> Option<&HashMap<ComponentId, Box<dyn MemoryModuleTrait>>> {
        self.owned_memory_modules.as_ref()
    }
}

impl<'a> crate::core::components::evaluation_context::TypeSafeMemoryProxy for MemoryProxy<'a> {
    fn read<T: MemoryData>(&self, port: &str, address: &str) -> Result<Option<T>, String> {
        self.read(port, address)
    }

    fn write<T: MemoryData>(&mut self, port: &str, address: &str, data: T) -> Result<(), String> {
        self.write(port, address, data)
    }
}

/// Owned memory proxy for parallel execution that doesn't hold lifetime parameters
/// This avoids the lifetime conflict when extracting updated memory components
pub struct OwnedMemoryProxy {
    /// Memory connections mapping: (component_id, port) -> memory_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,
    /// Current component ID for context
    component_id: ComponentId,
    /// Owned memory modules for parallel execution
    owned_memory_modules: HashMap<ComponentId, Box<dyn MemoryModuleTrait>>,
    /// Subset of memory component IDs for this specific component (parallel execution)
    memory_components_subset: Vec<ComponentId>,
    /// Memory delta tracking for parallel execution
    memory_delta: Option<MemoryDelta>,
}

impl OwnedMemoryProxy {
    /// Create a new owned memory proxy for parallel execution
    pub fn new(
        memory_connections: HashMap<(ComponentId, String), ComponentId>,
        component_id: ComponentId,
        owned_memory_modules: HashMap<ComponentId, Box<dyn MemoryModuleTrait>>,
        memory_subset: Vec<ComponentId>,
    ) -> Self {
        Self {
            memory_connections,
            component_id,
            owned_memory_modules,
            memory_components_subset: memory_subset,
            memory_delta: None,
        }
    }

    /// Create a new owned memory proxy with delta tracking for parallel execution
    pub fn new_with_delta_tracking(
        memory_connections: HashMap<(ComponentId, String), ComponentId>,
        component_id: ComponentId,
        owned_memory_modules: HashMap<ComponentId, Box<dyn MemoryModuleTrait>>,
        memory_subset: Vec<ComponentId>,
    ) -> Self {
        Self {
            memory_connections,
            component_id,
            owned_memory_modules,
            memory_components_subset: memory_subset,
            memory_delta: Some(MemoryDelta::new()),
        }
    }

    /// Read typed data from memory (reads from snapshot - previous cycle data)
    pub fn read<T: MemoryData>(&self, port: &str, address: &str) -> Result<Option<T>, String> {
        let mem_id = self
            .memory_connections
            .get(&(self.component_id.clone(), port.to_string()))
            .ok_or_else(|| format!("Memory port '{}' not connected for component '{}'", port, self.component_id))?;

        // Check if this memory component is allowed
        if !self.memory_components_subset.contains(mem_id) {
            return Err(format!("Memory component '{}' not in subset for component '{}'", mem_id, self.component_id));
        }

        if let Some(memory_module) = self.owned_memory_modules.get(mem_id) {
            if let Some(data_box) = memory_module.read_any(address) {
                // Try to downcast to the requested type
                if let Ok(typed_data) = data_box.downcast::<T>() {
                    Ok(Some(*typed_data))
                } else {
                    Err(format!("Type mismatch reading from memory address '{}' in memory '{}'", address, mem_id))
                }
            } else {
                Ok(None)
            }
        } else {
            Err(format!("Memory module '{}' not found in proxy registry", mem_id))
        }
    }

    /// Write typed data to memory (writes to current_state - affects next cycle)
    pub fn write<T: MemoryData>(&mut self, port: &str, address: &str, data: T) -> Result<(), String> {
        let mem_id = self
            .memory_connections
            .get(&(self.component_id.clone(), port.to_string()))
            .ok_or_else(|| format!("Memory port '{}' not connected for component '{}'", port, self.component_id))?;

        // Check if this memory component is allowed
        if !self.memory_components_subset.contains(mem_id) {
            return Err(format!("Memory component '{}' not in subset for component '{}'", mem_id, self.component_id));
        }

        // Clone data for delta tracking before consuming it
        let data_for_delta = data.clone();

        if let Some(memory_module) = self.owned_memory_modules.get_mut(mem_id) {
            let data_box: Box<dyn std::any::Any + Send> = Box::new(data);
            if memory_module.write_any(address, data_box) {
                // If delta tracking is enabled, record the write
                if let Some(ref mut delta) = self.memory_delta {
                    delta.record_write(
                        mem_id.clone(),
                        address.to_string(),
                        data_for_delta,
                        self.component_id.clone(),
                    );
                }
                Ok(())
            } else {
                Err(format!("Failed to write to memory address '{}' in memory '{}' - type mismatch", address, mem_id))
            }
        } else {
            Err(format!("Memory module '{}' not found in proxy registry", mem_id))
        }
    }

    /// Check if a memory port is connected
    pub fn is_connected(&self, port: &str) -> bool {
        self.memory_connections.contains_key(&(self.component_id.clone(), port.to_string()))
    }

    /// Get connected memory component ID for a port
    pub fn get_memory_id(&self, port: &str) -> Option<&ComponentId> {
        self.memory_connections.get(&(self.component_id.clone(), port.to_string()))
    }

    /// Extract updated memory components for merging back to main memory
    pub fn take_updated_memory_components(self) -> HashMap<ComponentId, Box<dyn MemoryModuleTrait>> {
        self.owned_memory_modules
    }

    /// Extract memory delta for parallel execution
    /// This method is used to get the memory changes that need to be applied to the main memory system
    pub fn take_memory_delta(&mut self) -> Option<MemoryDelta> {
        self.memory_delta.take()
    }

    /// Extract both memory delta and updated components (for destructuring)
    pub fn take_delta_and_components(mut self) -> (Option<MemoryDelta>, HashMap<ComponentId, Box<dyn MemoryModuleTrait>>) {
        let delta = self.memory_delta.take();
        let components = self.owned_memory_modules;
        (delta, components)
    }
}

impl crate::core::components::evaluation_context::TypeSafeMemoryProxy for OwnedMemoryProxy {
    fn read<T: MemoryData>(&self, port: &str, address: &str) -> Result<Option<T>, String> {
        self.read(port, address)
    }

    fn write<T: MemoryData>(&mut self, port: &str, address: &str, data: T) -> Result<(), String> {
        self.write(port, address, data)
    }
}

