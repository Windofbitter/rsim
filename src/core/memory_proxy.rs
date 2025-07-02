use super::component_module::{TypeSafeMemoryProxy, MemoryModuleTrait};
use super::state::MemoryData;
use super::types::ComponentId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Type-safe memory proxy for module-based components
pub struct TypeSafeCentralMemoryProxy {
    /// Memory modules
    memory_modules: HashMap<ComponentId, Rc<RefCell<Box<dyn MemoryModuleTrait>>>>,
    /// Memory connections mapping
    memory_connections: HashMap<(ComponentId, String), ComponentId>,
    /// Current component ID for context
    component_id: ComponentId,
}

impl TypeSafeCentralMemoryProxy {
    /// Create a new type-safe memory proxy
    pub fn new(
        memory_modules: &HashMap<ComponentId, Rc<RefCell<Box<dyn MemoryModuleTrait>>>>,
        memory_connections: &HashMap<(ComponentId, String), ComponentId>,
        component_id: ComponentId,
    ) -> Self {
        Self {
            memory_modules: memory_modules.clone(),
            memory_connections: memory_connections.clone(),
            component_id,
        }
    }
}

impl TypeSafeMemoryProxy for TypeSafeCentralMemoryProxy {
    /// Read typed data from memory
    fn read<T: MemoryData>(&self, port: &str, address: &str) -> Result<Option<T>, String> {
        let mem_id = self
            .memory_connections
            .get(&(self.component_id.clone(), port.to_string()))
            .ok_or_else(|| format!("Memory port '{}' not connected for component '{}'", port, self.component_id))?;

        if let Some(memory_module_ref) = self.memory_modules.get(mem_id) {
            let memory_module = memory_module_ref.borrow();
            if let Some(any_data) = memory_module.read_any(address) {
                // Attempt to downcast to the requested type
                match any_data.downcast::<T>() {
                    Ok(typed_data) => Ok(Some(*typed_data)),
                    Err(_) => Err(format!(
                        "Type mismatch: cannot convert memory data to {}",
                        std::any::type_name::<T>()
                    )),
                }
            } else {
                Ok(None)
            }
        } else {
            Err(format!("Memory component '{}' not found", mem_id))
        }
    }

    /// Write typed data to memory
    fn write<T: MemoryData>(&mut self, port: &str, address: &str, data: T) -> Result<(), String> {
        let mem_id = self
            .memory_connections
            .get(&(self.component_id.clone(), port.to_string()))
            .ok_or_else(|| format!("Memory port '{}' not connected for component '{}'", port, self.component_id))?;

        if let Some(memory_module_ref) = self.memory_modules.get(mem_id) {
            let mut memory_module = memory_module_ref.borrow_mut();
            let boxed_data: Box<dyn std::any::Any + Send> = Box::new(data);
            if memory_module.write_any(address, boxed_data) {
                Ok(())
            } else {
                Err(format!("Failed to write to memory '{}' at address '{}'", mem_id, address))
            }
        } else {
            Err(format!("Memory component '{}' not found", mem_id))
        }
    }

}

