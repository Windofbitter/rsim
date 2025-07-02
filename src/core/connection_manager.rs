use super::component_registry::ComponentRegistry;
use super::component_module::ComponentModule;
use super::connection_validator::ConnectionValidator;
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
        // Validate the memory connection
        ConnectionValidator::validate_memory_connection(registry, &proc_id, &port, &mem_id)?;

        // Check if this port is already connected to a memory component
        ConnectionValidator::check_memory_port_collision(&self.memory_connections, &proc_id, &port)?;

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

        // Validate the connection
        ConnectionValidator::validate_connection(registry, source_id, source_port, target_id, target_port)?;

        // Check for input port collision
        ConnectionValidator::check_input_port_collision(&self.connections, target_id, target_port)?;

        self.connections.entry(source).or_default().push(target);
        Ok(())
    }



    /// Validate all connections in the manager
    pub fn validate_all_connections(&self, registry: &ComponentRegistry) -> Result<(), String> {
        // Validate regular connections
        ConnectionValidator::validate_all_connections(registry, &self.connections)?;

        // Validate memory connections
        ConnectionValidator::validate_all_memory_connections(registry, &self.memory_connections)?;


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