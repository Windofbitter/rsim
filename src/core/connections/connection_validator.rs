use crate::core::builder::simulation_builder::ComponentInstance;
use crate::core::connections::port_validator::PortValidator;
use crate::core::types::ComponentId;
use std::collections::HashMap;

/// Centralized connection validation logic for the new direct API
pub struct ConnectionValidator;

impl ConnectionValidator {
    /// Create a new connection validator
    pub fn new() -> Self {
        Self
    }

    /// Validate a connection between components with direct component access
    pub fn validate_connection_direct(
        source_component: &ComponentInstance,
        source_port: &str,
        target_component: &ComponentInstance,
        target_port: &str,
    ) -> Result<(), String> {
        // Validate source port exists
        PortValidator::validate_source_port(source_component, source_port)?;

        // Validate target port exists
        PortValidator::validate_target_port(target_component, target_port)?;

        Ok(())
    }

    /// Validate a memory connection with direct component access
    pub fn validate_memory_connection_direct(
        component: &ComponentInstance,
        port: &str,
    ) -> Result<(), String> {
        PortValidator::validate_memory_port(component, port)
    }

    /// Check if an input port is already connected (prevents multiple drivers)
    pub fn check_input_port_collision(
        connections: &HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
        target_id: &ComponentId,
        target_port: &str,
    ) -> Result<(), String> {
        for existing_targets in connections.values() {
            for (existing_target_id, existing_target_port) in existing_targets {
                if existing_target_id == target_id && existing_target_port == target_port {
                    return Err(format!(
                        "Input port '{}' on component '{}' is already connected. Multiple drivers not allowed.",
                        target_port, target_id
                    ));
                }
            }
        }
        Ok(())
    }

    /// Check if a memory port is already connected
    pub fn check_memory_port_collision(
        memory_connections: &HashMap<(ComponentId, String), ComponentId>,
        proc_id: &ComponentId,
        port: &str,
    ) -> Result<(), String> {
        if let Some(existing_mem_id) = memory_connections.get(&(proc_id.clone(), port.to_string())) {
            return Err(format!(
                "Memory port '{}' on component '{}' is already connected to memory '{}'",
                port, proc_id, existing_mem_id
            ));
        }
        Ok(())
    }
}