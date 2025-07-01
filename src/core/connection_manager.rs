use super::component::{ProcessingComponent, MemoryComponent, ProbeComponent};
use super::types::ComponentId;
use std::collections::HashMap;

pub struct ConnectionManager {
    pub processing_components: HashMap<ComponentId, Box<dyn ProcessingComponent>>,
    pub memory_components: HashMap<ComponentId, Box<dyn MemoryComponent>>,
    pub probe_components: HashMap<ComponentId, Box<dyn ProbeComponent>>,
    
    // Port connections: (source_id, port) -> Vec<(target_id, port)>
    pub connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    
    // Memory connections: (component_id, port) -> memory_component_id
    pub memory_connections: HashMap<(ComponentId, String), ComponentId>,
    
    // Probe connections: (source_id, port) -> Vec<probe_id>
    pub probes: HashMap<(ComponentId, String), Vec<ComponentId>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            processing_components: HashMap::new(),
            memory_components: HashMap::new(),
            probe_components: HashMap::new(),
            connections: HashMap::new(),
            memory_connections: HashMap::new(),
            probes: HashMap::new(),
        }
    }

    pub fn register_processing(&mut self, component: Box<dyn ProcessingComponent>) {
        let id = component.component_id().clone();
        self.processing_components.insert(id, component);
    }

    pub fn register_memory(&mut self, component: Box<dyn MemoryComponent>) {
        let id = component.component_id().clone();
        self.memory_components.insert(id, component);
    }

    pub fn register_probe(&mut self, component: Box<dyn ProbeComponent>) {
        let id = component.component_id().clone();
        self.probe_components.insert(id, component);
    }

    pub fn connect(&mut self, source: (ComponentId, String), target: (ComponentId, String)) -> Result<(), String> {
        // Validate that both components exist
        if !self.processing_components.contains_key(&source.0) && !self.memory_components.contains_key(&source.0) {
            return Err(format!("Source component {} does not exist", source.0));
        }
        if !self.processing_components.contains_key(&target.0) && !self.memory_components.contains_key(&target.0) {
            return Err(format!("Target component {} does not exist", target.0));
        }
        
        self.connections.entry(source).or_default().push(target);
        Ok(())
    }

    pub fn connect_memory(&mut self, proc_id: ComponentId, port: String, mem_id: ComponentId) -> Result<(), String> {
        // Validate that processing component exists
        if !self.processing_components.contains_key(&proc_id) {
            return Err(format!("Processing component {} does not exist", proc_id));
        }
        
        // Validate that memory component exists
        if !self.memory_components.contains_key(&mem_id) {
            return Err(format!("Memory component {} does not exist", mem_id));
        }
        
        // Check if port is already connected to memory
        if self.memory_connections.contains_key(&(proc_id.clone(), port.clone())) {
            return Err(format!("Port ({}, {}) already connected to memory", proc_id, port));
        }
        
        self.memory_connections.insert((proc_id, port), mem_id);
        Ok(())
    }

    pub fn add_probe(&mut self, source_port: (ComponentId, String), probe_id: ComponentId) -> Result<(), String> {
        // Validate that source component exists
        if !self.processing_components.contains_key(&source_port.0) && !self.memory_components.contains_key(&source_port.0) {
            return Err(format!("Source component {} does not exist", source_port.0));
        }
        
        // Validate that probe component exists
        if !self.probe_components.contains_key(&probe_id) {
            return Err(format!("Probe component {} does not exist", probe_id));
        }
        
        self.probes.entry(source_port).or_default().push(probe_id);
        Ok(())
    }
}