use crate::core::components::module::ComponentModule;
use crate::core::components::state::MemoryData;
use crate::core::components::traits::{Component, MemoryComponent};
use crate::core::execution::cycle_engine::CycleEngine;
use crate::core::types::{ComponentId, OutputPort, InputPort, MemoryPort};
use std::collections::HashMap;

/// Simplified component instance for direct module usage
pub struct ComponentInstance {
    pub id: ComponentId,
    pub module: ComponentModule,
}

/// Imperative API for creating and configuring simulations with direct module creation
/// 
/// This simplified builder removes the component manager registration system
/// and allows direct addition of component modules.
pub struct Simulation {
    /// Created component instances mapped by ID
    components: HashMap<ComponentId, ComponentInstance>,
    /// Port connections: (source_id, source_port) -> Vec<(target_id, target_port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    /// Memory connections: (component_id, port) -> memory_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,
    /// Counter for automatic ID generation
    id_counter: std::sync::atomic::AtomicU64,
}

impl Simulation {
    /// Create a new simulation
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            connections: HashMap::new(),
            memory_connections: HashMap::new(),
            id_counter: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Add a component directly using the Component trait
    pub fn add_component<T: Component>(&mut self, _component: T) -> ComponentId {
        let counter = self.id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let type_name = std::any::type_name::<T>();
        let clean_type_name = type_name.split("::").last().unwrap_or(type_name);
        let id = format!("{}{}", clean_type_name, counter);
        
        let module = T::into_module();
        let component_id = ComponentId::new(id, type_name.to_string());
        
        let instance = ComponentInstance {
            id: component_id.clone(),
            module: ComponentModule::Processing(module),
        };
        
        self.components.insert(component_id.clone(), instance);
        component_id
    }

    /// Add a memory component directly using the MemoryComponent trait
    pub fn add_memory_component<T: MemoryComponent + MemoryData>(&mut self, _component: T) -> ComponentId {
        let counter = self.id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let type_name = std::any::type_name::<T>();
        let clean_type_name = type_name.split("::").last().unwrap_or(type_name);
        let id = format!("{}{}", clean_type_name, counter);
        
        let module = T::into_memory_module();
        let component_id = ComponentId::new(id, type_name.to_string());
        
        let instance = ComponentInstance {
            id: component_id.clone(),
            module: ComponentModule::Memory(Box::new(module)),
        };
        
        self.components.insert(component_id.clone(), instance);
        component_id
    }

    // Old auto methods removed - new add_component methods auto-generate IDs

    /// Connect two component ports using port handles
    pub fn connect(&mut self, source: OutputPort, target: InputPort) -> Result<(), String> {
        let source_id = source.component_id().clone();
        let source_port = source.port_name().to_string();
        let target_id = target.component_id().clone();
        let target_port = target.port_name().to_string();
        
        // Validate that both components exist
        if !self.components.contains_key(&source_id) {
            return Err(format!("Source component '{}' not found", source_id));
        }
        if !self.components.contains_key(&target_id) {
            return Err(format!("Target component '{}' not found", target_id));
        }
        
        // Validate that the source component has the specified output port
        let source_component = self.components.get(&source_id).unwrap();
        if let Some(processor) = source_component.module.as_processing() {
            if !processor.has_output_port(&source_port) {
                return Err(format!("Component '{}' does not have output port '{}'", 
                                  source_id, source_port));
            }
        } else {
            return Err(format!("Source component '{}' is not a processing component", source_id));
        }
        
        // Validate that the target component has the specified input port
        let target_component = self.components.get(&target_id).unwrap();
        if let Some(processor) = target_component.module.as_processing() {
            if !processor.has_input_port(&target_port) {
                return Err(format!("Component '{}' does not have input port '{}'", 
                                  target_id, target_port));
            }
        } else {
            return Err(format!("Target component '{}' is not a processing component", target_id));
        }
        
        // Validate 1-to-1 connections: Check if output port is already connected
        if self.connections.contains_key(&(source_id.clone(), source_port.clone())) {
            return Err(format!("Output port '{}' on component '{}' is already connected", 
                              source_port, source_id));
        }
        
        // Validate 1-to-1 connections: Check if input port is already connected
        for targets in self.connections.values() {
            if targets.iter().any(|(tid, tport)| tid == &target_id && tport == &target_port) {
                return Err(format!("Input port '{}' on component '{}' is already connected", 
                                  target_port, target_id));
            }
        }
        
        // If all validations pass, make the connection (single target per output)
        self.connections.insert((source_id, source_port), vec![(target_id, target_port)]);
        Ok(())
    }

    /// Connect two component ports using port handles (alias for connect)
    pub fn connect_component(&mut self, source: OutputPort, target: InputPort) -> Result<(), String> {
        self.connect(source, target)
    }

    /// Connect a component memory port to a memory component using output port handle
    pub fn connect_memory(&mut self, component_port: OutputPort, memory_id: ComponentId) -> Result<(), String> {
        let comp_id = component_port.component_id().clone();
        let port_name = component_port.port_name().to_string();
        
        // Validate that the memory component exists
        if !self.components.contains_key(&memory_id) {
            return Err(format!("Memory component '{}' not found", memory_id));
        }
        
        // Validate that the memory component is actually a memory component
        let memory_component = self.components.get(&memory_id).unwrap();
        if !memory_component.module.is_memory() {
            return Err(format!("Component '{}' is not a memory component", memory_id));
        }
        
        // Validate that the source component exists
        if !self.components.contains_key(component_port.component_id()) {
            return Err(format!("Source component '{}' not found", component_port.component_id()));
        }
        
        // Check for duplicate connections (each memory port can only be connected once)
        let connection_key = (comp_id.clone(), port_name.clone());
        if self.memory_connections.contains_key(&connection_key) {
            return Err(format!("Memory port '{}' on component '{}' is already connected", 
                              port_name, comp_id));
        }
        
        // Validate that the source component has the specified port
        let source_component = self.components.get(component_port.component_id()).unwrap();
        if let Some(processor) = source_component.module.as_processing() {
            if !processor.has_memory_port(&port_name) {
                return Err(format!("Component '{}' does not have memory port '{}'", 
                                  component_port.component_id(), port_name));
            }
        } else {
            return Err(format!("Only processing components can connect to memory components"));
        }
        
        // If all validations pass, make the connection
        self.memory_connections.insert(connection_key, memory_id);
        Ok(())
    }

    /// Connect a component memory port to a memory component using memory port handle
    pub fn connect_memory_port(&mut self, component_port: MemoryPort, memory_id: ComponentId) -> Result<(), String> {
        let comp_id = component_port.component_id().clone();
        let port_name = component_port.port_name().to_string();
        
        // Validate that the memory component exists
        if !self.components.contains_key(&memory_id) {
            return Err(format!("Memory component '{}' not found", memory_id));
        }
        
        // Validate that the memory component is actually a memory component
        let memory_component = self.components.get(&memory_id).unwrap();
        if !memory_component.module.is_memory() {
            return Err(format!("Component '{}' is not a memory component", memory_id));
        }
        
        // Validate that the source component exists
        if !self.components.contains_key(component_port.component_id()) {
            return Err(format!("Source component '{}' not found", component_port.component_id()));
        }
        
        // Check for duplicate connections (each memory port can only be connected once)
        let connection_key = (comp_id.clone(), port_name.clone());
        if self.memory_connections.contains_key(&connection_key) {
            return Err(format!("Memory port '{}' on component '{}' is already connected", 
                              port_name, comp_id));
        }
        
        // Validate that the source component has the specified port
        let source_component = self.components.get(component_port.component_id()).unwrap();
        if let Some(processor) = source_component.module.as_processing() {
            if !processor.has_memory_port(&port_name) {
                return Err(format!("Component '{}' does not have memory port '{}'", 
                                  component_port.component_id(), port_name));
            }
        } else {
            return Err(format!("Source component '{}' is not a processing component", component_port.component_id()));
        }
        
        // If all validations pass, make the connection
        self.memory_connections.insert(connection_key, memory_id);
        Ok(())
    }

    /// Build the simulation into a CycleEngine
    pub fn build(self) -> Result<CycleEngine, String> {
        // Validate connections
        self.validate_connections()?;
        
        // Create and configure cycle engine
        let mut cycle_engine = CycleEngine::new();
        
        // Add all components to the cycle engine
        for (_, instance) in self.components {
            cycle_engine.register_component_instance(instance)?;
        }
        
        // Add all connections
        for ((source_id, source_port), targets) in self.connections {
            for (target_id, target_port) in targets {
                cycle_engine.connect((source_id.clone(), source_port.clone()), (target_id, target_port))?;
            }
        }
        
        // Add all memory connections
        for ((component_id, port), memory_id) in self.memory_connections {
            cycle_engine.connect_memory((component_id, port), memory_id)?;
        }
        
        Ok(cycle_engine)
    }

    // parse_port_reference removed - using port handles directly now

    /// Validate all connections
    fn validate_connections(&self) -> Result<(), String> {
        use crate::core::connections::port_validator::PortValidator;
        
        for ((source_id, source_port), targets) in &self.connections {
            // Check source component exists
            let source_component = self.components.get(source_id)
                .ok_or_else(|| format!("Source component '{}' not found", source_id))?;
            
            // Check source port exists and is output
            PortValidator::validate_source_port(source_component, source_port)?;
            
            for (target_id, target_port) in targets {
                // Check target component exists
                let target_component = self.components.get(target_id)
                    .ok_or_else(|| format!("Target component '{}' not found", target_id))?;
                
                // Check target port exists and is input
                PortValidator::validate_target_port(target_component, target_port)?;
            }
        }
        
        // Validate memory connections
        for ((comp_id, port), _memory_id) in &self.memory_connections {
            let component = self.components.get(comp_id)
                .ok_or_else(|| format!("Component '{}' not found", comp_id))?;
            
            PortValidator::validate_memory_port(component, port)?;
        }
        
        Ok(())
    }

    /// Get all component IDs
    pub fn component_ids(&self) -> Vec<&ComponentId> {
        self.components.keys().collect()
    }

    /// Get component by ID
    pub fn get_component(&self, id: &ComponentId) -> Option<&ComponentInstance> {
        self.components.get(id)
    }

    /// Check if component exists
    pub fn has_component(&self, id: &ComponentId) -> bool {
        self.components.contains_key(id)
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for fluent simulation building
pub trait SimulationExt {
    /// Start building a simulation
    fn simulation() -> Simulation {
        Simulation::new()
    }
}

// Blanket implementation
impl SimulationExt for () {}

