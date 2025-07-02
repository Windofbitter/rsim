use super::component_manager::{ComponentManager, ComponentInstance};
use super::component_module::ComponentModule;
use super::cycle_engine::CycleEngine;
use super::connection_validator::ConnectionValidator;
use super::types::{ComponentId, OutputPort, InputPort, MemoryPort};
use std::collections::HashMap;

/// Imperative API for creating and configuring simulations
pub struct Simulation {
    /// Component manager for creating instances
    component_manager: ComponentManager,
    /// Created component instances  
    components: HashMap<ComponentId, ComponentInstance>,
    /// Port connections: (source_id, source_port) -> Vec<(target_id, target_port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    /// Memory connections: (component_id, port) -> memory_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,
}

impl Simulation {
    /// Create a new simulation
    pub fn new() -> Self {
        Self {
            component_manager: ComponentManager::new(),
            components: HashMap::new(),
            connections: HashMap::new(),
            memory_connections: HashMap::new(),
        }
    }

    /// Register a component module template
    pub fn register_module(&mut self, name: &str, module: ComponentModule) -> Result<(), String> {
        self.component_manager.register_module(name, module)
    }

    /// Create a component with automatic ID generation
    pub fn create_component(&mut self, module_name: &str) -> Result<ComponentId, String> {
        let instance = self.component_manager.create_component_auto_id(module_name)?;
        let id = instance.id().clone();
        self.components.insert(id.clone(), instance);
        Ok(id)
    }

    /// Create a component with a specific ID
    pub fn create_component_with_id(&mut self, module_name: &str, id_string: String) -> Result<ComponentId, String> {
        let temp_id = ComponentId::new(id_string.clone(), module_name.to_string());
        if self.components.contains_key(&temp_id) {
            return Err(format!("Component with ID '{}' already exists", id_string));
        }
        
        let instance = self.component_manager.create_component(module_name, id_string)?;
        let component_id = instance.id().clone();
        self.components.insert(component_id.clone(), instance);
        Ok(component_id)
    }

    /// Create multiple components of the same type
    pub fn create_components(&mut self, module_name: &str, count: usize) -> Result<Vec<ComponentId>, String> {
        let instances = self.component_manager.create_components(module_name, count)?;
        let mut component_ids = Vec::new();
        for instance in instances {
            let id = instance.id().clone();
            component_ids.push(id.clone());
            self.components.insert(id, instance);
        }
        Ok(component_ids)
    }

    /// Create components with custom prefix
    pub fn create_components_with_prefix(&mut self, module_name: &str, prefix: &str, count: usize) -> Result<Vec<ComponentId>, String> {
        let instances = self.component_manager.create_components_with_prefix(module_name, prefix, count)?;
        let mut component_ids = Vec::new();
        for instance in instances {
            let id = instance.id().clone();
            component_ids.push(id.clone());
            self.components.insert(id, instance);
        }
        Ok(component_ids)
    }

    /// Connect two component ports using port handles
    pub fn connect(&mut self, output: OutputPort, input: InputPort) -> Result<(), String> {
        let source_id = output.component_id();
        let source_port = output.port_name();
        let target_id = input.component_id();
        let target_port = input.port_name();

        // Validate components exist
        if !self.components.contains_key(source_id) {
            return Err(format!("Source component '{}' not found", source_id));
        }
        if !self.components.contains_key(target_id) {
            return Err(format!("Target component '{}' not found", target_id));
        }

        // Validate ports exist
        let source_component = self.components.get(source_id)
            .ok_or_else(|| format!("Source component '{}' not found", source_id))?;
        let target_component = self.components.get(target_id)
            .ok_or_else(|| format!("Target component '{}' not found", target_id))?;
        
        ConnectionValidator::validate_connection_direct(source_component, source_port, target_component, target_port)?;

        // Check for input port collision
        for targets in self.connections.values() {
            for (existing_target_id, existing_target_port) in targets {
                if existing_target_id == target_id && existing_target_port == target_port {
                    return Err(format!(
                        "Input port '{}' on component '{}' is already connected. Multiple drivers not allowed.",
                        target_port, target_id
                    ));
                }
            }
        }

        let source_key = (source_id.clone(), source_port.to_string());
        let target_tuple = (target_id.clone(), target_port.to_string());
        
        self.connections.entry(source_key).or_default().push(target_tuple);
        Ok(())
    }

    /// Connect a component memory port to a memory component using port handles
    pub fn connect_memory(&mut self, memory_port: MemoryPort, memory_component: &ComponentId) -> Result<(), String> {
        let component_id = memory_port.component_id();
        let port = memory_port.port_name();
        
        // Validate components exist
        if !self.components.contains_key(component_id) {
            return Err(format!("Component '{}' not found", component_id));
        }
        if !self.components.contains_key(memory_component) {
            return Err(format!("Memory component '{}' not found", memory_component));
        }

        // Validate the component has the memory port
        let component = self.components.get(component_id)
            .ok_or_else(|| format!("Component '{}' not found", component_id))?;
        ConnectionValidator::validate_memory_connection_direct(component, port)?;

        // Validate the target is a memory component
        let memory_instance = &self.components[memory_component];
        if !memory_instance.is_memory() {
            return Err(format!("Component '{}' is not a memory component", memory_component));
        }

        // Check if this port is already connected
        let port_key = (component_id.clone(), port.to_string());
        if self.memory_connections.contains_key(&port_key) {
            return Err(format!(
                "Memory port '{}' on component '{}' is already connected",
                port, component_id
            ));
        }

        self.memory_connections.insert(port_key, memory_component.clone());
        Ok(())
    }


    /// Validate all connections and required ports
    pub fn validate_connections(&self) -> Result<(), String> {
        // Check that all required input ports are connected
        for (component_id, instance) in &self.components {
            if let Some(proc_module) = instance.module.as_processing() {
                for port_spec in &proc_module.input_ports {
                    if port_spec.required {
                        let port_connected = self.connections.values()
                            .any(|targets| targets.iter()
                                .any(|(target_id, target_port)| 
                                    target_id == component_id && target_port == &port_spec.name));
                        
                        if !port_connected {
                            return Err(format!(
                                "Required input port '{}' on component '{}' is not connected",
                                port_spec.name, component_id
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate that required ports are connected for specific components
    pub fn validate_required_ports(&self) -> Result<(), String> {
        for (component_id, instance) in &self.components {
            if let Some(proc_module) = instance.module.as_processing() {
                let connected_ports: Vec<String> = self.connections.values()
                    .flatten()
                    .filter_map(|(target_id, target_port)| {
                        if target_id == component_id {
                            Some(target_port.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                self.component_manager.validate_component_ports(
                    proc_module.name.as_str(),
                    &connected_ports
                )?;
            }
        }

        Ok(())
    }

    /// Build a CycleEngine from the configured components and connections
    pub fn build(self) -> Result<CycleEngine, String> {
        // Final validation
        self.validate_connections()?;
        self.validate_required_ports()?;

        // Create the engine and register components
        let mut engine = CycleEngine::new();

        // Register all components
        for (_, instance) in self.components {
            engine.register_component(instance)?;
        }

        // Set up connections
        for ((source_id, source_port), targets) in self.connections {
            for (target_id, target_port) in targets {
                engine.connect(
                    (source_id.clone(), source_port.clone()),
                    (target_id, target_port),
                )?;
            }
        }

        // Set up memory connections
        for ((component_id, port), memory_id) in self.memory_connections {
            engine.connect_memory(component_id, port, memory_id)?;
        }

        // Build execution order
        engine.build_execution_order()?;

        Ok(engine)
    }

    /// Get component statistics
    pub fn component_stats(&self) -> SimulationStats {
        let mut processing_count = 0;
        let mut memory_count = 0;

        for instance in self.components.values() {
            if instance.is_processing() {
                processing_count += 1;
            } else if instance.is_memory() {
                memory_count += 1;
            }
        }

        SimulationStats {
            total_components: self.components.len(),
            processing_components: processing_count,
            memory_components: memory_count,
            total_connections: self.connections.len(),
            memory_connections: self.memory_connections.len(),
        }
    }

    /// Get all component IDs
    pub fn component_ids(&self) -> Vec<&ComponentId> {
        self.components.keys().collect()
    }

    /// Get a component instance by ID
    pub fn get_component(&self, id: &ComponentId) -> Option<&ComponentInstance> {
        self.components.get(id)
    }

}

/// Statistics about the simulation configuration
#[derive(Debug, Clone)]
pub struct SimulationStats {
    pub total_components: usize,
    pub processing_components: usize,
    pub memory_components: usize,
    pub total_connections: usize,
    pub memory_connections: usize,
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait to provide additional convenience methods
pub trait SimulationExt {
    /// Create multiple components with their IDs
    fn create_components_batch(&mut self, specs: Vec<(&str, Option<&str>)>) -> Result<Vec<ComponentId>, String>;
}

impl SimulationExt for Simulation {
    fn create_components_batch(&mut self, specs: Vec<(&str, Option<&str>)>) -> Result<Vec<ComponentId>, String> {
        let mut component_ids = Vec::new();
        for (module_name, component_id) in specs {
            let id = match component_id {
                Some(id) => self.create_component_with_id(module_name, id.to_string())?,
                None => self.create_component(module_name)?,
            };
            component_ids.push(id);
        }
        Ok(component_ids)
    }
}