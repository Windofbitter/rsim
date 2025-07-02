use super::component_registry::ComponentRegistry;
use super::component_module::ComponentModule;
use super::types::ComponentId;
use std::collections::HashMap;

/// Manages all connections between components and validates connection rules
pub struct ConnectionManager {
    /// Memory connections: (component_id, port) -> memory_component_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,

    /// Port connections: (source_id, port) -> Vec<(target_id, port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            memory_connections: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    /// Connect a processing component memory port to a memory component
    pub fn connect_memory(
        &mut self,
        registry: &ComponentRegistry,
        proc_id: ComponentId,
        port: String,
        mem_id: ComponentId,
    ) -> Result<(), String> {
        // Validate that the processing component exists
        if !registry.has_processing_component(&proc_id) {
            return Err(format!("Processing component '{}' not found", proc_id));
        }

        // Validate that the memory component exists
        if !registry.has_memory_component(&mem_id) {
            return Err(format!("Memory component '{}' not found", mem_id));
        }

        // Validate that the port exists on the processing component
        if let Some(component) = registry.get_component(&proc_id) {
            if let ComponentModule::Processing(proc_module) = &component.module {
                if !proc_module.has_memory_port(&port) {
                    return Err(format!(
                        "Memory port '{}' not found on component '{}'. Valid ports: {:?}",
                        port, proc_id, proc_module.memory_port_names()
                    ));
                }
            }
        }

        // Check if this port is already connected to a memory component
        if let Some(existing_mem_id) = self
            .memory_connections
            .get(&(proc_id.clone(), port.clone()))
        {
            return Err(format!(
                "Memory port '{}' on component '{}' is already connected to memory '{}'",
                port, proc_id, existing_mem_id
            ));
        }

        self.memory_connections.insert((proc_id, port), mem_id);
        Ok(())
    }

    /// Connect two component ports
    pub fn connect(
        &mut self,
        registry: &ComponentRegistry,
        source: (ComponentId, String),
        target: (ComponentId, String),
    ) -> Result<(), String> {
        let (source_id, source_port) = &source;
        let (target_id, target_port) = &target;

        // Validate that source component exists
        if !registry.has_component(source_id) {
            return Err(format!("Source component '{}' not found", source_id));
        }

        // Validate that target component exists
        if !registry.has_component(target_id) {
            return Err(format!("Target component '{}' not found", target_id));
        }

        // Validate source port exists
        self.validate_source_port(registry, source_id, source_port)?;

        // Validate target port exists
        self.validate_target_port(registry, target_id, target_port)?;

        // Check for input port collision
        for existing_targets in self.connections.values() {
            for (existing_target_id, existing_target_port) in existing_targets {
                if existing_target_id == target_id && existing_target_port == target_port {
                    return Err(format!(
                        "Input port '{}' on component '{}' is already connected. Multiple drivers not allowed.",
                        target_port, target_id
                    ));
                }
            }
        }

        self.connections.entry(source).or_default().push(target);
        Ok(())
    }


    /// Validate that a component has the specified output port
    fn validate_source_port(
        &self,
        registry: &ComponentRegistry,
        component_id: &ComponentId,
        port: &str,
    ) -> Result<(), String> {
        if let Some(component) = registry.get_component(component_id) {
            match &component.module {
                ComponentModule::Processing(proc_module) => {
                    if !proc_module.has_output_port(port) {
                        return Err(format!(
                            "Output port '{}' not found on processing component '{}'. Valid ports: {:?}",
                            port, component_id, proc_module.output_port_names()
                        ));
                    }
                },
                ComponentModule::Memory(_) => {
                    // Memory components have a standard output port "out"
                    if port != "out" {
                        return Err(format!(
                            "Output port '{}' not found on memory component '{}'. Valid port: 'out'",
                            port, component_id
                        ));
                    }
                },
            }
        }
        Ok(())
    }

    /// Validate that a component has the specified input port
    fn validate_target_port(
        &self,
        registry: &ComponentRegistry,
        component_id: &ComponentId,
        port: &str,
    ) -> Result<(), String> {
        if let Some(component) = registry.get_component(component_id) {
            match &component.module {
                ComponentModule::Processing(proc_module) => {
                    if !proc_module.has_input_port(port) {
                        return Err(format!(
                            "Input port '{}' not found on processing component '{}'. Valid ports: {:?}",
                            port, component_id, proc_module.input_port_names()
                        ));
                    }
                },
                ComponentModule::Memory(_) => {
                    // Memory components have a standard input port "in"
                    if port != "in" {
                        return Err(format!(
                            "Input port '{}' not found on memory component '{}'. Valid port: 'in'",
                            port, component_id
                        ));
                    }
                },
            }
        }
        Ok(())
    }

    /// Validate all connections in the manager
    pub fn validate_all_connections(&self, registry: &ComponentRegistry) -> Result<(), String> {
        // Validate regular connections
        for ((source_id, source_port), targets) in &self.connections {
            self.validate_source_port(registry, source_id, source_port)?;
            for (target_id, target_port) in targets {
                self.validate_target_port(registry, target_id, target_port)?;
            }
        }

        // Validate memory connections
        for ((component_id, port), memory_id) in &self.memory_connections {
            if !registry.has_component(component_id) {
                return Err(format!("Component '{}' in memory connection not found", component_id));
            }
            if !registry.has_memory_component(memory_id) {
                return Err(format!("Memory component '{}' in memory connection not found", memory_id));
            }

            // Validate the memory port exists
            if let Some(component) = registry.get_component(component_id) {
                if let ComponentModule::Processing(proc_module) = &component.module {
                    if !proc_module.has_memory_port(port) {
                        return Err(format!(
                            "Memory port '{}' not found on component '{}'",
                            port, component_id
                        ));
                    }
                }
            }
        }


        Ok(())
    }

    /// Check if all required input ports are connected
    pub fn validate_required_connections(&self, registry: &ComponentRegistry) -> Result<(), String> {
        for (component_id, instance) in registry.components() {
            if let ComponentModule::Processing(proc_module) = &instance.module {
                for port_spec in &proc_module.input_ports {
                    if port_spec.required {
                        // Check if this port is connected
                        let is_connected = self.connections.values()
                            .any(|targets| targets.iter()
                                .any(|(target_id, target_port)| 
                                    target_id == component_id && target_port == &port_spec.name));
                        
                        if !is_connected {
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

    /// Get connection statistics
    pub fn connection_stats(&self) -> ConnectionStats {
        let total_targets: usize = self.connections.values()
            .map(|targets| targets.len())
            .sum();

        ConnectionStats {
            total_connections: total_targets,
            memory_connections: self.memory_connections.len(),
            unique_sources: self.connections.len(),
        }
    }

    // Getters for connection data
    pub fn memory_connections(&self) -> &HashMap<(ComponentId, String), ComponentId> {
        &self.memory_connections
    }

    pub fn connections(&self) -> &HashMap<(ComponentId, String), Vec<(ComponentId, String)>> {
        &self.connections
    }


    /// Remove all connections involving a specific component
    pub fn remove_component_connections(&mut self, component_id: &ComponentId) {
        // Remove as source
        self.connections.retain(|(source_id, _), _| source_id != component_id);
        self.memory_connections.retain(|(comp_id, _), _| comp_id != component_id);

        // Remove as target
        for targets in self.connections.values_mut() {
            targets.retain(|(target_id, _)| target_id != component_id);
        }

        // Remove as memory target
        self.memory_connections.retain(|_, memory_id| memory_id != component_id);
    }

    /// Clear all connections
    pub fn clear(&mut self) {
        self.connections.clear();
        self.memory_connections.clear();
    }
}

/// Statistics about connections
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub memory_connections: usize,
    pub unique_sources: usize,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}