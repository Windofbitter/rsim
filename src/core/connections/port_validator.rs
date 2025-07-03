use crate::core::builder::simulation_builder::ComponentInstance;
use crate::core::components::types::PortType;

/// Port validation utilities for components
pub struct PortValidator;

impl PortValidator {
    /// Validate that a component has the specified output port
    pub fn validate_source_port(
        component: &ComponentInstance,
        port: &str,
    ) -> Result<(), String> {
        // Check if port exists in module's port list
        let has_output_port = component.module.ports()
            .iter()
            .any(|(name, port_type)| name == port && *port_type == PortType::Output);
        
        if !has_output_port {
            let ports = component.module.ports();
            let valid_outputs: Vec<&str> = ports
                .iter()
                .filter(|(_, port_type)| *port_type == PortType::Output)
                .map(|(name, _)| name.as_str())
                .collect();
            
            return Err(format!(
                "Output port '{}' not found on component '{}'. Valid output ports: {:?}",
                port, component.id.id(), valid_outputs
            ));
        }
        Ok(())
    }

    /// Validate that a component has the specified input port
    pub fn validate_target_port(
        component: &ComponentInstance,
        port: &str,
    ) -> Result<(), String> {
        // Check if port exists in module's port list
        let has_input_port = component.module.ports()
            .iter()
            .any(|(name, port_type)| name == port && *port_type == PortType::Input);
        
        if !has_input_port {
            let ports = component.module.ports();
            let valid_inputs: Vec<&str> = ports
                .iter()
                .filter(|(_, port_type)| *port_type == PortType::Input)
                .map(|(name, _)| name.as_str())
                .collect();
            
            return Err(format!(
                "Input port '{}' not found on component '{}'. Valid input ports: {:?}",
                port, component.id.id(), valid_inputs
            ));
        }
        Ok(())
    }

    /// Validate that a processing component has the specified memory port
    pub fn validate_memory_port(
        component: &ComponentInstance,
        port: &str,
    ) -> Result<(), String> {
        if !component.module.is_processing() {
            return Err(format!(
                "Component '{}' is not a processing component and cannot have memory ports",
                component.id.id()
            ));
        }
        
        // Check if port exists in module's port list as memory port
        let has_memory_port = component.module.ports()
            .iter()
            .any(|(name, port_type)| name == port && *port_type == PortType::Memory);
        
        if !has_memory_port {
            let ports = component.module.ports();
            let valid_memory_ports: Vec<&str> = ports
                .iter()
                .filter(|(_, port_type)| *port_type == PortType::Memory)
                .map(|(name, _)| name.as_str())
                .collect();
            
            return Err(format!(
                "Memory port '{}' not found on component '{}'. Valid memory ports: {:?}",
                port, component.id.id(), valid_memory_ports
            ));
        }
        Ok(())
    }

    // Registry-based validation methods removed - using direct component instances now
}