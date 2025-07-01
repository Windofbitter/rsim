use super::component::{MemoryComponent, ProbeComponent, ProcessingComponent};
use super::types::ComponentId;
use std::cell::RefCell;
use std::collections::HashMap;

/// Manages registration and storage of all component types
pub struct ComponentRegistry {
    processing_components: HashMap<ComponentId, Box<dyn ProcessingComponent>>,
    memory_components: HashMap<ComponentId, RefCell<Box<dyn MemoryComponent>>>,
    probe_components: HashMap<ComponentId, Box<dyn ProbeComponent>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            processing_components: HashMap::new(),
            memory_components: HashMap::new(),
            probe_components: HashMap::new(),
        }
    }

    pub fn register_processing(&mut self, component: Box<dyn ProcessingComponent>) {
        let id = component.component_id().clone();
        self.processing_components.insert(id, component);
    }

    pub fn register_memory(&mut self, component: Box<dyn MemoryComponent>) {
        let id = component.component_id().clone();
        self.memory_components.insert(id, RefCell::new(component));
    }

    pub fn register_probe(&mut self, component: Box<dyn ProbeComponent>) {
        let id = component.component_id().clone();
        self.probe_components.insert(id, component);
    }

    // Getters for component collections
    pub fn processing_components(&self) -> &HashMap<ComponentId, Box<dyn ProcessingComponent>> {
        &self.processing_components
    }

    pub fn memory_components(&self) -> &HashMap<ComponentId, RefCell<Box<dyn MemoryComponent>>> {
        &self.memory_components
    }

    pub fn probe_components(&mut self) -> &mut HashMap<ComponentId, Box<dyn ProbeComponent>> {
        &mut self.probe_components
    }

    pub fn probe_components_ref(&self) -> &HashMap<ComponentId, Box<dyn ProbeComponent>> {
        &self.probe_components
    }

    // Component existence checks
    pub fn has_processing_component(&self, id: &ComponentId) -> bool {
        self.processing_components.contains_key(id)
    }

    pub fn has_memory_component(&self, id: &ComponentId) -> bool {
        self.memory_components.contains_key(id)
    }

    pub fn has_probe_component(&self, id: &ComponentId) -> bool {
        self.probe_components.contains_key(id)
    }

    pub fn has_component(&self, id: &ComponentId) -> bool {
        self.has_processing_component(id) 
            || self.has_memory_component(id) 
            || self.has_probe_component(id)
    }
}