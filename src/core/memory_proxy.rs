use super::component::{EngineMemoryProxy, Event, MemoryComponent, MemoryError};
use super::types::ComponentId;
use std::cell::RefCell;
use std::collections::HashMap;

/// Engine's centralized memory proxy that manages memory access for components
pub struct CentralMemoryProxy<'a> {
    memory_components: &'a HashMap<ComponentId, RefCell<Box<dyn MemoryComponent>>>,
    memory_connections: &'a HashMap<(ComponentId, String), ComponentId>,
}

impl<'a> CentralMemoryProxy<'a> {
    pub fn new(
        memory_components: &'a HashMap<ComponentId, RefCell<Box<dyn MemoryComponent>>>,
        memory_connections: &'a HashMap<(ComponentId, String), ComponentId>,
    ) -> Self {
        Self {
            memory_components,
            memory_connections,
        }
    }
}

impl<'a> EngineMemoryProxy for CentralMemoryProxy<'a> {
    fn read(&self, component_id: &ComponentId, port: &str, address: &str) -> Result<Option<Event>, MemoryError> {
        let mem_id = self
            .memory_connections
            .get(&(component_id.clone(), port.to_string()))
            .ok_or_else(|| MemoryError::InvalidPort(format!("{}:{}", component_id, port)))?;
        
        let memory_ref = self
            .memory_components
            .get(mem_id)
            .ok_or_else(|| MemoryError::MemoryNotFound(mem_id.clone()))?;
        
        Ok(memory_ref.borrow().read_snapshot(address))
    }

    fn write(&mut self, component_id: &ComponentId, port: &str, address: &str, data: Event) -> Result<(), MemoryError> {
        let mem_id = self
            .memory_connections
            .get(&(component_id.clone(), port.to_string()))
            .ok_or_else(|| MemoryError::InvalidPort(format!("{}:{}", component_id, port)))?;
        
        let memory_ref = self
            .memory_components
            .get(mem_id)
            .ok_or_else(|| MemoryError::MemoryNotFound(mem_id.clone()))?;
        
        let success = memory_ref.borrow_mut().write(address, data);
        if success {
            Ok(())
        } else {
            Err(MemoryError::OperationFailed(format!("Write failed to {}:{}", mem_id, address)))
        }
    }
}