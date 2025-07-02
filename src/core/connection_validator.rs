use super::component_registry::{ComponentRegistry, ComponentType};
use super::component_manager::ComponentInstance;
use super::component_module::ComponentModule;
use super::port_validator::PortValidator;
use super::types::ComponentId;
use std::collections::HashMap;

/// Centralized connection validation logic for all connection managers
pub struct ConnectionValidator;

impl ConnectionValidator {
    /// Validate a regular port-to-port connection
    pub fn validate_connection(
        registry: &ComponentRegistry,
        source_id: &ComponentId,
        source_port: &str,
        target_id: &ComponentId,
        target_port: &str,
    ) -> Result<(), String> {
        // Validate that source component exists
        if !registry.has_component(source_id) {
            return Err(format!("Source component '{}' not found", source_id));
        }

        // Validate that target component exists
        if !registry.has_component(target_id) {
            return Err(format!("Target component '{}' not found", target_id));
        }

        // Validate source port exists
        PortValidator::validate_source_port_with_registry(registry, source_id, source_port)?;

        // Validate target port exists
        PortValidator::validate_target_port_with_registry(registry, target_id, target_port)?;

        Ok(())
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

    /// Validate a memory connection
    pub fn validate_memory_connection(
        registry: &ComponentRegistry,
        proc_id: &ComponentId,
        port: &str,
        mem_id: &ComponentId,
    ) -> Result<(), String> {
        // Validate that the processing component exists
        if !registry.has_component_of_type(proc_id, ComponentType::Processing) {
            return Err(format!("Processing component '{}' not found", proc_id));
        }

        // Validate that the memory component exists
        if !registry.has_component_of_type(mem_id, ComponentType::Memory) {
            return Err(format!("Memory component '{}' not found", mem_id));
        }

        // Validate that the port exists on the processing component
        if let Some(component) = registry.get_component(proc_id) {
            if let ComponentModule::Processing(proc_module) = &component.module {
                if !proc_module.has_memory_port(port) {
                    return Err(format!(
                        "Memory port '{}' not found on component '{}'. Valid ports: {:?}",
                        port, proc_id, proc_module.memory_port_names()
                    ));
                }
            }
        }

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

    /// Validate all connections in a connection map
    pub fn validate_all_connections(
        registry: &ComponentRegistry,
        connections: &HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    ) -> Result<(), String> {
        for ((source_id, source_port), targets) in connections {
            PortValidator::validate_source_port_with_registry(registry, source_id, source_port)?;
            for (target_id, target_port) in targets {
                PortValidator::validate_target_port_with_registry(registry, target_id, target_port)?;
            }
        }
        Ok(())
    }

    /// Validate all memory connections in a memory connection map
    pub fn validate_all_memory_connections(
        registry: &ComponentRegistry,
        memory_connections: &HashMap<(ComponentId, String), ComponentId>,
    ) -> Result<(), String> {
        for ((component_id, port), memory_id) in memory_connections {
            if !registry.has_component(component_id) {
                return Err(format!("Component '{}' in memory connection not found", component_id));
            }
            if !registry.has_component_of_type(memory_id, ComponentType::Memory) {
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
}