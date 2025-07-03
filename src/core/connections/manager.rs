use crate::core::types::ComponentId;
use std::collections::HashMap;

/// Simplified connection manager for the new direct API
/// 
/// This manager handles regular port connections and memory connections
/// in the new simulation system.
pub struct ConnectionManager {
    /// Regular port connections: (source_id, source_port) -> Vec<(target_id, target_port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    /// Memory connections: (component_id, port) -> memory_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            memory_connections: HashMap::new(),
        }
    }

    /// Add a regular port connection
    pub fn add_connection(
        &mut self,
        source_id: ComponentId,
        source_port: String,
        target_id: ComponentId,
        target_port: String,
    ) -> Result<(), String> {
        self.connections
            .entry((source_id, source_port))
            .or_insert_with(Vec::new)
            .push((target_id, target_port));
        Ok(())
    }

    /// Add a memory connection
    pub fn add_memory_connection(
        &mut self,
        component_id: ComponentId,
        port: String,
        memory_id: ComponentId,
    ) -> Result<(), String> {
        if self.memory_connections.contains_key(&(component_id.clone(), port.clone())) {
            return Err(format!(
                "Memory port '{}' on component '{}' is already connected",
                port, component_id
            ));
        }

        self.memory_connections.insert((component_id, port), memory_id);
        Ok(())
    }

    /// Get all regular connections
    pub fn connections(&self) -> &HashMap<(ComponentId, String), Vec<(ComponentId, String)>> {
        &self.connections
    }

    /// Get all memory connections
    pub fn memory_connections(&self) -> &HashMap<(ComponentId, String), ComponentId> {
        &self.memory_connections
    }

    /// Get targets for a source port
    pub fn get_targets(&self, source_id: &ComponentId, source_port: &str) -> Option<&Vec<(ComponentId, String)>> {
        self.connections.get(&(source_id.clone(), source_port.to_string()))
    }

    /// Get memory connection for a component port
    pub fn get_memory_connection(&self, component_id: &ComponentId, port: &str) -> Option<&ComponentId> {
        self.memory_connections.get(&(component_id.clone(), port.to_string()))
    }

    /// Check if a port has any connections
    pub fn is_connected(&self, component_id: &ComponentId, port: &str) -> bool {
        // Check regular connections (both as source and target)
        let has_output_connections = self.connections.contains_key(&(component_id.clone(), port.to_string()));
        
        let has_input_connections = self.connections.values()
            .any(|targets| targets.iter()
                .any(|(target_id, target_port)| target_id == component_id && target_port == port));

        // Check memory connections
        let has_memory_connection = self.memory_connections.contains_key(&(component_id.clone(), port.to_string()));

        has_output_connections || has_input_connections || has_memory_connection
    }

    /// Get connection statistics
    pub fn stats(&self) -> ConnectionStats {
        ConnectionStats {
            regular_connections: self.connections.len(),
            memory_connections: self.memory_connections.len(),
            total_targets: self.connections.values().map(|v| v.len()).sum(),
        }
    }
}

/// Connection statistics for debugging
#[derive(Debug)]
pub struct ConnectionStats {
    pub regular_connections: usize,
    pub memory_connections: usize,
    pub total_targets: usize,
}