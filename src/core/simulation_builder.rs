use super::component_manager::{ComponentManager, ComponentInstance};
use super::component_module::ComponentModule;
use super::cycle_engine::CycleEngine;
use super::types::ComponentId;
use std::collections::HashMap;

/// Builder pattern for creating and configuring simulations with fluent API
pub struct SimulationBuilder {
    /// Component manager for creating instances
    component_manager: ComponentManager,
    /// Created component instances  
    components: HashMap<ComponentId, ComponentInstance>,
    /// Port connections: (source_id, source_port) -> Vec<(target_id, target_port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    /// Memory connections: (component_id, port) -> memory_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,
    /// Probe connections: (source_id, port) -> Vec<probe_id>
    probe_connections: HashMap<(ComponentId, String), Vec<ComponentId>>,
}

impl SimulationBuilder {
    /// Create a new simulation builder
    pub fn new() -> Self {
        Self {
            component_manager: ComponentManager::new(),
            components: HashMap::new(),
            connections: HashMap::new(),
            memory_connections: HashMap::new(),
            probe_connections: HashMap::new(),
        }
    }

    /// Register a component module template
    pub fn register_module(mut self, name: &str, module: ComponentModule) -> Result<Self, String> {
        self.component_manager.register_module(name, module)?;
        Ok(self)
    }

    /// Create a component with automatic ID generation
    pub fn create_component(mut self, module_name: &str) -> Result<Self, String> {
        let instance = self.component_manager.create_component_auto_id(module_name)?;
        let id = instance.id().clone();
        self.components.insert(id, instance);
        Ok(self)
    }

    /// Create a component with a specific ID
    pub fn create_component_with_id(mut self, module_name: &str, component_id: ComponentId) -> Result<Self, String> {
        if self.components.contains_key(&component_id) {
            return Err(format!("Component with ID '{}' already exists", component_id));
        }
        
        let instance = self.component_manager.create_component(module_name, component_id.clone())?;
        self.components.insert(component_id, instance);
        Ok(self)
    }

    /// Create multiple components of the same type
    pub fn create_components(mut self, module_name: &str, count: usize) -> Result<Self, String> {
        let instances = self.component_manager.create_components(module_name, count)?;
        for instance in instances {
            let id = instance.id().clone();
            self.components.insert(id, instance);
        }
        Ok(self)
    }

    /// Create components with custom prefix
    pub fn create_components_with_prefix(mut self, module_name: &str, prefix: &str, count: usize) -> Result<Self, String> {
        let instances = self.component_manager.create_components_with_prefix(module_name, prefix, count)?;
        for instance in instances {
            let id = instance.id().clone();
            self.components.insert(id, instance);
        }
        Ok(self)
    }

    /// Connect two component ports
    pub fn connect(mut self, source: (&str, &str), target: (&str, &str)) -> Result<Self, String> {
        let (source_id, source_port) = source;
        let (target_id, target_port) = target;

        // Validate components exist
        if !self.components.contains_key(source_id) {
            return Err(format!("Source component '{}' not found", source_id));
        }
        if !self.components.contains_key(target_id) {
            return Err(format!("Target component '{}' not found", target_id));
        }

        // Validate ports exist
        self.validate_source_port(source_id, source_port)?;
        self.validate_target_port(target_id, target_port)?;

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

        let source_key = (source_id.to_string(), source_port.to_string());
        let target_tuple = (target_id.to_string(), target_port.to_string());
        
        self.connections.entry(source_key).or_default().push(target_tuple);
        Ok(self)
    }

    /// Connect a component memory port to a memory component
    pub fn connect_memory(mut self, component_id: &str, port: &str, memory_id: &str) -> Result<Self, String> {
        // Validate components exist
        if !self.components.contains_key(component_id) {
            return Err(format!("Component '{}' not found", component_id));
        }
        if !self.components.contains_key(memory_id) {
            return Err(format!("Memory component '{}' not found", memory_id));
        }

        // Validate the component has the memory port
        self.validate_memory_port(component_id, port)?;

        // Validate the target is a memory component
        let memory_instance = &self.components[memory_id];
        if !memory_instance.is_memory() {
            return Err(format!("Component '{}' is not a memory component", memory_id));
        }

        // Check if this port is already connected
        let port_key = (component_id.to_string(), port.to_string());
        if self.memory_connections.contains_key(&port_key) {
            return Err(format!(
                "Memory port '{}' on component '{}' is already connected",
                port, component_id
            ));
        }

        self.memory_connections.insert(port_key, memory_id.to_string());
        Ok(self)
    }

    /// Connect a component output to a probe
    pub fn connect_probe(mut self, source: (&str, &str), probe_id: &str) -> Result<Self, String> {
        let (source_id, source_port) = source;

        // Validate components exist
        if !self.components.contains_key(source_id) {
            return Err(format!("Source component '{}' not found", source_id));
        }
        if !self.components.contains_key(probe_id) {
            return Err(format!("Probe component '{}' not found", probe_id));
        }

        // Validate source port exists
        self.validate_source_port(source_id, source_port)?;

        // Validate the target is a probe component
        let probe_instance = &self.components[probe_id];
        if !probe_instance.is_probe() {
            return Err(format!("Component '{}' is not a probe component", probe_id));
        }

        let source_key = (source_id.to_string(), source_port.to_string());
        self.probe_connections.entry(source_key).or_default().push(probe_id.to_string());
        Ok(self)
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

        // Set up probe connections
        for ((source_id, source_port), probe_ids) in self.probe_connections {
            for probe_id in probe_ids {
                engine.connect_probe((source_id.clone(), source_port.clone()), probe_id)?;
            }
        }

        // Build execution order
        engine.build_execution_order()?;

        Ok(engine)
    }

    /// Get component statistics
    pub fn component_stats(&self) -> SimulationStats {
        let mut processing_count = 0;
        let mut memory_count = 0;
        let mut probe_count = 0;

        for instance in self.components.values() {
            if instance.is_processing() {
                processing_count += 1;
            } else if instance.is_memory() {
                memory_count += 1;
            } else if instance.is_probe() {
                probe_count += 1;
            }
        }

        SimulationStats {
            total_components: self.components.len(),
            processing_components: processing_count,
            memory_components: memory_count,
            probe_components: probe_count,
            total_connections: self.connections.len(),
            memory_connections: self.memory_connections.len(),
            probe_connections: self.probe_connections.len(),
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

    /// Helper method to validate source port exists
    fn validate_source_port(&self, component_id: &str, port: &str) -> Result<(), String> {
        let instance = &self.components[component_id];
        
        match &instance.module {
            ComponentModule::Processing(proc_module) => {
                if !proc_module.has_output_port(port) {
                    return Err(format!(
                        "Output port '{}' not found on processing component '{}'. Valid ports: {:?}",
                        port, component_id, proc_module.output_port_names()
                    ));
                }
            },
            ComponentModule::Memory(_) => {
                // Memory components typically have a standard output port "out"
                if port != "out" {
                    return Err(format!(
                        "Output port '{}' not found on memory component '{}'. Valid port: 'out'",
                        port, component_id
                    ));
                }
            },
            ComponentModule::Probe(_) => {
                return Err(format!("Probe components don't have output ports"));
            },
        }
        
        Ok(())
    }

    /// Helper method to validate target port exists
    fn validate_target_port(&self, component_id: &str, port: &str) -> Result<(), String> {
        let instance = &self.components[component_id];
        
        match &instance.module {
            ComponentModule::Processing(proc_module) => {
                if !proc_module.has_input_port(port) {
                    return Err(format!(
                        "Input port '{}' not found on processing component '{}'. Valid ports: {:?}",
                        port, component_id, proc_module.input_port_names()
                    ));
                }
            },
            ComponentModule::Memory(_) => {
                // Memory components typically have a standard input port "in"
                if port != "in" {
                    return Err(format!(
                        "Input port '{}' not found on memory component '{}'. Valid port: 'in'",
                        port, component_id
                    ));
                }
            },
            ComponentModule::Probe(_) => {
                return Err(format!("Probe components don't have input ports for direct connections"));
            },
        }
        
        Ok(())
    }

    /// Helper method to validate memory port exists
    fn validate_memory_port(&self, component_id: &str, port: &str) -> Result<(), String> {
        let instance = &self.components[component_id];
        
        if let ComponentModule::Processing(proc_module) = &instance.module {
            if !proc_module.has_memory_port(port) {
                return Err(format!(
                    "Memory port '{}' not found on component '{}'. Valid ports: {:?}",
                    port, component_id, proc_module.memory_port_names()
                ));
            }
        } else {
            return Err(format!(
                "Component '{}' is not a processing component and cannot have memory ports",
                component_id
            ));
        }
        
        Ok(())
    }
}

/// Statistics about the simulation configuration
#[derive(Debug, Clone)]
pub struct SimulationStats {
    pub total_components: usize,
    pub processing_components: usize,
    pub memory_components: usize,
    pub probe_components: usize,
    pub total_connections: usize,
    pub memory_connections: usize,
    pub probe_connections: usize,
}

impl Default for SimulationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait to provide additional builder methods
pub trait SimulationBuilderExt {
    /// Chain multiple component creation calls
    fn with_components(self, specs: Vec<(&str, Option<&str>)>) -> Result<Self, String>
    where
        Self: Sized;

    /// Chain multiple connection calls
    fn with_connections(self, connections: Vec<((&str, &str), (&str, &str))>) -> Result<Self, String>
    where
        Self: Sized;
}

impl SimulationBuilderExt for SimulationBuilder {
    fn with_components(mut self, specs: Vec<(&str, Option<&str>)>) -> Result<Self, String> {
        for (module_name, component_id) in specs {
            self = match component_id {
                Some(id) => self.create_component_with_id(module_name, id.to_string())?,
                None => self.create_component(module_name)?,
            };
        }
        Ok(self)
    }

    fn with_connections(mut self, connections: Vec<((&str, &str), (&str, &str))>) -> Result<Self, String> {
        for (source, target) in connections {
            self = self.connect(source, target)?;
        }
        Ok(self)
    }
}