use crate::core::components::module::ComponentModule;
use crate::core::components::manager::ComponentInstance;
use crate::core::components::registry::ComponentRegistry;
use crate::core::types::ComponentId;

/// Port validation utilities for components
pub struct PortValidator;

impl PortValidator {
    /// Validate that a component has the specified output port
    pub fn validate_source_port(
        component: &ComponentInstance,
        port: &str,
    ) -> Result<(), String> {
        match &component.module {
            ComponentModule::Processing(proc_module) => {
                if !proc_module.has_output_port(port) {
                    return Err(format!(
                        "Output port '{}' not found on processing component '{}'. Valid ports: {:?}",
                        port, component.id(), proc_module.output_port_names()
                    ));
                }
            },
            ComponentModule::Memory(_) => {
                // Memory components have a standard output port "out"
                if port != "out" {
                    return Err(format!(
                        "Output port '{}' not found on memory component '{}'. Valid port: 'out'",
                        port, component.id()
                    ));
                }
            },
        }
        Ok(())
    }

    /// Validate that a component has the specified input port
    pub fn validate_target_port(
        component: &ComponentInstance,
        port: &str,
    ) -> Result<(), String> {
        match &component.module {
            ComponentModule::Processing(proc_module) => {
                if !proc_module.has_input_port(port) {
                    return Err(format!(
                        "Input port '{}' not found on processing component '{}'. Valid ports: {:?}",
                        port, component.id(), proc_module.input_port_names()
                    ));
                }
            },
            ComponentModule::Memory(_) => {
                // Memory components have a standard input port "in"
                if port != "in" {
                    return Err(format!(
                        "Input port '{}' not found on memory component '{}'. Valid port: 'in'",
                        port, component.id()
                    ));
                }
            },
        }
        Ok(())
    }

    /// Validate that a processing component has the specified memory port
    pub fn validate_memory_port(
        component: &ComponentInstance,
        port: &str,
    ) -> Result<(), String> {
        if let ComponentModule::Processing(proc_module) = &component.module {
            if !proc_module.has_memory_port(port) {
                return Err(format!(
                    "Memory port '{}' not found on component '{}'. Valid ports: {:?}",
                    port, component.id(), proc_module.memory_port_names()
                ));
            }
        } else {
            return Err(format!(
                "Component '{}' is not a processing component and cannot have memory ports",
                component.id()
            ));
        }
        Ok(())
    }

    /// Validate source port with registry lookup
    pub fn validate_source_port_with_registry(
        registry: &ComponentRegistry,
        component_id: &ComponentId,
        port: &str,
    ) -> Result<(), String> {
        if let Some(component) = registry.get_component(component_id) {
            Self::validate_source_port(component, port)
        } else {
            Err(format!("Component '{}' not found", component_id))
        }
    }

    /// Validate target port with registry lookup
    pub fn validate_target_port_with_registry(
        registry: &ComponentRegistry,
        component_id: &ComponentId,
        port: &str,
    ) -> Result<(), String> {
        if let Some(component) = registry.get_component(component_id) {
            Self::validate_target_port(component, port)
        } else {
            Err(format!("Component '{}' not found", component_id))
        }
    }

    /// Validate memory port with registry lookup
    pub fn validate_memory_port_with_registry(
        registry: &ComponentRegistry,
        component_id: &ComponentId,
        port: &str,
    ) -> Result<(), String> {
        if let Some(component) = registry.get_component(component_id) {
            Self::validate_memory_port(component, port)
        } else {
            Err(format!("Component '{}' not found", component_id))
        }
    }
}