#[cfg(test)]
mod tests {
    use crate::core::component_manager::ComponentInstance;
    use crate::core::component_module::{ComponentModule, ProcessorModule, MemoryModule, PortSpec, PortType};
    use crate::core::port_validator::PortValidator;

    #[test]
    fn test_port_validator_exists() {
        // Simple test to verify the PortValidator struct exists and has the expected methods
        
        // Create a dummy processing component for testing
        let input_ports = vec![PortSpec {
            name: "in1".to_string(),
            port_type: PortType::Input,
            required: true,
            description: Some("Test input".to_string()),
        }];
        
        let output_ports = vec![PortSpec {
            name: "out1".to_string(),
            port_type: PortType::Output,
            required: false,
            description: Some("Test output".to_string()),
        }];
        
        let memory_ports = vec![PortSpec {
            name: "mem1".to_string(),
            port_type: PortType::Memory,
            required: false,
            description: Some("Test memory".to_string()),
        }];
        
        fn dummy_evaluate(_ctx: &crate::core::component_module::EvaluationContext, _outputs: &mut crate::core::typed_values::TypedOutputMap) -> Result<(), String> {
            Ok(())
        }
        
        let proc_module = ProcessorModule::new(
            "test_proc",
            input_ports,
            output_ports,
            memory_ports,
            dummy_evaluate,
        );
        
        let component = ComponentInstance {
            id: "test_comp".to_string(),
            module: ComponentModule::Processing(proc_module),
            state: None,
        };
        
        // Test valid output port
        let result = PortValidator::validate_source_port(&component, "out1");
        assert!(result.is_ok(), "Valid source port should be accepted");
        
        // Test invalid output port
        let result = PortValidator::validate_source_port(&component, "invalid_out");
        assert!(result.is_err(), "Invalid source port should be rejected");
        assert!(result.unwrap_err().contains("Output port 'invalid_out' not found"));
        
        // Test valid input port
        let result = PortValidator::validate_target_port(&component, "in1");
        assert!(result.is_ok(), "Valid target port should be accepted");
        
        // Test invalid input port
        let result = PortValidator::validate_target_port(&component, "invalid_in");
        assert!(result.is_err(), "Invalid target port should be rejected");
        assert!(result.unwrap_err().contains("Input port 'invalid_in' not found"));
        
        // Test valid memory port
        let result = PortValidator::validate_memory_port(&component, "mem1");
        assert!(result.is_ok(), "Valid memory port should be accepted");
        
        // Test invalid memory port
        let result = PortValidator::validate_memory_port(&component, "invalid_mem");
        assert!(result.is_err(), "Invalid memory port should be rejected");
        assert!(result.unwrap_err().contains("Memory port 'invalid_mem' not found"));
    }

    #[test]
    fn test_memory_component_validation() {
        // Test memory component port validation
        let memory_module = MemoryModule::<i64>::new("test_mem");
        let mem_component = ComponentInstance {
            id: "test_mem".to_string(),
            module: ComponentModule::Memory(Box::new(memory_module)),
            state: None,
        };
        
        // Memory components should accept "out" as source port
        let result = PortValidator::validate_source_port(&mem_component, "out");
        assert!(result.is_ok(), "Memory component should accept 'out' as source port");
        
        // Memory components should reject other source ports
        let result = PortValidator::validate_source_port(&mem_component, "other");
        assert!(result.is_err(), "Memory component should reject non-'out' source ports");
        
        // Memory components should accept "in" as target port
        let result = PortValidator::validate_target_port(&mem_component, "in");
        assert!(result.is_ok(), "Memory component should accept 'in' as target port");
        
        // Memory components should reject other target ports
        let result = PortValidator::validate_target_port(&mem_component, "other");
        assert!(result.is_err(), "Memory component should reject non-'in' target ports");
        
        // Memory components should not have memory ports
        let result = PortValidator::validate_memory_port(&mem_component, "any_port");
        assert!(result.is_err(), "Memory components should not have memory ports");
        assert!(result.unwrap_err().contains("is not a processing component"));
    }
}