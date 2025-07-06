use crate::core::components::state::MemoryData;
use crate::core::components::module::MemoryModuleTrait;
use crate::core::types::ComponentId;
use std::collections::HashMap;

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
}