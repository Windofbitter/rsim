use super::component_manager::ComponentInstance;
use super::component_module::{ComponentModule, MemoryModuleTrait};
use super::types::ComponentId;
use std::cell::RefCell;
use std::collections::HashMap;

/// Manages registration and storage of all module-based component types
pub struct ComponentRegistry {
    /// All component instances
    components: HashMap<ComponentId, ComponentInstance>,
    /// Memory modules (separated for efficient access)
    memory_modules: HashMap<ComponentId, RefCell<Box<dyn MemoryModuleTrait>>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            memory_modules: HashMap::new(),
        }
    }

    /// Register a component instance
    pub fn register_component(&mut self, instance: ComponentInstance) -> Result<(), String> {
        let id = instance.id().clone();
        
        if self.components.contains_key(&id) {
            return Err(format!("Component with ID '{}' already exists", id));
        }
        
        // Separate memory modules for efficient access
        if instance.is_memory() {
            if let ComponentModule::Memory(memory_module) = &instance.module {
                self.memory_modules.insert(id.clone(), RefCell::new(memory_module.clone_module()));
            }
        }
        
        self.components.insert(id, instance);
        Ok(())
    }

    /// Get a component by ID
    pub fn get_component(&self, id: &ComponentId) -> Option<&ComponentInstance> {
        self.components.get(id)
    }

    /// Get a mutable component by ID
    pub fn get_component_mut(&mut self, id: &ComponentId) -> Option<&mut ComponentInstance> {
        self.components.get_mut(id)
    }

    /// Get all components
    pub fn components(&self) -> &HashMap<ComponentId, ComponentInstance> {
        &self.components
    }

    /// Get all components mutably
    pub fn components_mut(&mut self) -> &mut HashMap<ComponentId, ComponentInstance> {
        &mut self.components
    }

    /// Get memory modules
    pub fn memory_modules(&self) -> &HashMap<ComponentId, RefCell<Box<dyn MemoryModuleTrait>>> {
        &self.memory_modules
    }

    /// Get processing components
    pub fn processing_components(&self) -> HashMap<ComponentId, &ComponentInstance> {
        self.components
            .iter()
            .filter(|(_, instance)| instance.is_processing())
            .map(|(id, instance)| (id.clone(), instance))
            .collect()
    }

    /// Get memory component instances
    pub fn memory_component_instances(&self) -> HashMap<ComponentId, &ComponentInstance> {
        self.components
            .iter()
            .filter(|(_, instance)| instance.is_memory())
            .map(|(id, instance)| (id.clone(), instance))
            .collect()
    }

    /// Get probe components
    pub fn probe_components(&self) -> HashMap<ComponentId, &ComponentInstance> {
        self.components
            .iter()
            .filter(|(_, instance)| instance.is_probe())
            .map(|(id, instance)| (id.clone(), instance))
            .collect()
    }

    /// Get mutable probe components
    pub fn probe_components_mut(&mut self) -> HashMap<ComponentId, &mut ComponentInstance> {
        self.components
            .iter_mut()
            .filter(|(_, instance)| instance.is_probe())
            .map(|(id, instance)| (id.clone(), instance))
            .collect()
    }

    /// Check if a component exists
    pub fn has_component(&self, id: &ComponentId) -> bool {
        self.components.contains_key(id)
    }

    /// Check if a processing component exists
    pub fn has_processing_component(&self, id: &ComponentId) -> bool {
        self.components
            .get(id)
            .map(|instance| instance.is_processing())
            .unwrap_or(false)
    }

    /// Check if a memory component exists
    pub fn has_memory_component(&self, id: &ComponentId) -> bool {
        self.components
            .get(id)
            .map(|instance| instance.is_memory())
            .unwrap_or(false)
    }

    /// Check if a probe component exists
    pub fn has_probe_component(&self, id: &ComponentId) -> bool {
        self.components
            .get(id)
            .map(|instance| instance.is_probe())
            .unwrap_or(false)
    }

    /// Get all component IDs
    pub fn component_ids(&self) -> Vec<&ComponentId> {
        self.components.keys().collect()
    }

    /// Get processing component IDs
    pub fn processing_component_ids(&self) -> Vec<ComponentId> {
        self.components
            .iter()
            .filter(|(_, instance)| instance.is_processing())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get memory component IDs
    pub fn memory_component_ids(&self) -> Vec<ComponentId> {
        self.components
            .iter()
            .filter(|(_, instance)| instance.is_memory())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get probe component IDs
    pub fn probe_component_ids(&self) -> Vec<ComponentId> {
        self.components
            .iter()
            .filter(|(_, instance)| instance.is_probe())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Remove a component by ID
    pub fn remove_component(&mut self, id: &ComponentId) -> Option<ComponentInstance> {
        // Remove from memory modules if it's a memory component
        self.memory_modules.remove(id);
        // Remove from main components
        self.components.remove(id)
    }

    /// Get component count by type
    pub fn component_counts(&self) -> ComponentCounts {
        let mut processing = 0;
        let mut memory = 0;
        let mut probe = 0;

        for instance in self.components.values() {
            if instance.is_processing() {
                processing += 1;
            } else if instance.is_memory() {
                memory += 1;
            } else if instance.is_probe() {
                probe += 1;
            }
        }

        ComponentCounts {
            total: self.components.len(),
            processing,
            memory,
            probe,
        }
    }

    /// Clear all components
    pub fn clear(&mut self) {
        self.components.clear();
        self.memory_modules.clear();
    }

    /// Check registry consistency
    pub fn validate_consistency(&self) -> Result<(), String> {
        // Check that all memory modules have corresponding component instances
        for memory_id in self.memory_modules.keys() {
            if !self.components.contains_key(memory_id) {
                return Err(format!("Memory module '{}' has no corresponding component instance", memory_id));
            }
            
            let instance = &self.components[memory_id];
            if !instance.is_memory() {
                return Err(format!("Component '{}' has memory module but is not a memory component", memory_id));
            }
        }

        // Check that all memory component instances have corresponding memory modules
        for (id, instance) in &self.components {
            if instance.is_memory() && !self.memory_modules.contains_key(id) {
                return Err(format!("Memory component '{}' has no corresponding memory module", id));
            }
        }

        Ok(())
    }
}

/// Statistics about component counts
#[derive(Debug, Clone)]
pub struct ComponentCounts {
    pub total: usize,
    pub processing: usize,
    pub memory: usize,
    pub probe: usize,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}