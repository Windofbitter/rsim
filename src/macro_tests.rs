//! Test the macro system with simple components

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::core::values::traits::{EventInputs, EventOutputs};

    // Test the impl_component macro
    struct TestAdder;
    
    impl_component!(TestAdder, "TestAdder", {
        inputs: [a, b],
        outputs: [sum],
        memory: [],
        react: |ctx, outputs| {
            let a: i32 = ctx.inputs.get("a").unwrap_or_default();
            let b: i32 = ctx.inputs.get("b").unwrap_or_default();
            outputs.set("sum", a + b)?;
            Ok(())
        }
    });

    // Test the impl_memory_component macro
    #[derive(Clone)]
    struct TestBuffer {
        data: i32,
    }
    
    impl crate::core::components::state::MemoryData for TestBuffer {}
    
    impl crate::core::components::Cycle for TestBuffer {
        type Output = i32;
        
        fn cycle(&mut self) -> Option<Self::Output> {
            Some(self.data)
        }
    }
    
    impl_memory_component!(TestBuffer, {
        input: input,
        output: output
    });

    // Test the component! macro for complete component definition
    component! {
        name: TestCalculator,
        component_name: "TestCalculator",
        inputs: [x, y],
        outputs: [result],
        memory: [],
        react: |ctx, outputs| {
            let x: f64 = ctx.inputs.get("x").unwrap_or_default();
            let y: f64 = ctx.inputs.get("y").unwrap_or_default();
            outputs.set("result", x * y)?;
            Ok(())
        }
    }

    #[test]
    fn test_impl_component_ports() {
        let ports = TestAdder::define_ports();
        assert_eq!(ports.len(), 3);
        
        // Check that ports are correctly defined
        assert!(ports.contains(&("a".to_string(), crate::core::components::PortType::Input)));
        assert!(ports.contains(&("b".to_string(), crate::core::components::PortType::Input)));
        assert!(ports.contains(&("sum".to_string(), crate::core::components::PortType::Output)));
    }

    #[test]
    fn test_impl_memory_component_ports() {
        let ports = TestBuffer::define_ports();
        assert_eq!(ports.len(), 2);
        
        // Check that ports are correctly defined
        assert!(ports.contains(&("input".to_string(), crate::core::components::PortType::Input)));
        assert!(ports.contains(&("output".to_string(), crate::core::components::PortType::Output)));
    }

    #[test]
    fn test_component_module_creation() {
        // This test verifies that the into_module() method is generated correctly
        let module = TestAdder::into_module();
        assert_eq!(module.name, "TestAdder");
        assert_eq!(module.input_ports.len(), 2);
        assert_eq!(module.output_ports.len(), 1);
        assert_eq!(module.memory_ports.len(), 0);
    }

    #[test]
    fn test_complete_component_macro() {
        // Test the component! macro
        let ports = TestCalculator::define_ports();
        assert_eq!(ports.len(), 3);
        
        let module = TestCalculator::into_module();
        assert_eq!(module.name, "TestCalculator");
    }

    #[test]
    fn test_port_macros() {
        // Test the port definition macros
        let input_ports = input_ports![a, b, c];
        assert_eq!(input_ports.len(), 3);
        
        let output_ports = output_ports![result];
        assert_eq!(output_ports.len(), 1);
        
        let memory_ports = memory_ports![state, buffer];
        assert_eq!(memory_ports.len(), 2);
    }

    #[test]
    fn test_port_definitions_macro() {
        let ports = port_definitions![
            inputs: [a, b],
            outputs: [sum],
            memory: [state],
        ];
        
        assert_eq!(ports.len(), 4);
        assert!(ports.contains(&("a".to_string(), crate::core::components::types::PortType::Input)));
        assert!(ports.contains(&("b".to_string(), crate::core::components::types::PortType::Input)));
        assert!(ports.contains(&("sum".to_string(), crate::core::components::types::PortType::Output)));
        assert!(ports.contains(&("state".to_string(), crate::core::components::types::PortType::Memory)));
    }
}