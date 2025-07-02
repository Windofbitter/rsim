// Tests for CycleEngine functionality
#[cfg(test)]
mod tests {
    use crate::core::{
        execution::cycle_engine::CycleEngine,
        components::manager::ComponentManager,
        components::module::{ComponentModule, ProcessorModule, PortSpec},
        types::ComponentId,
    };

    #[test]
    fn test_execution_order_builder_integration() {
        // Test that CycleEngine properly uses ExecutionOrderBuilder
        let mut engine = CycleEngine::new();
        
        // Create a simple component manager and processor module
        let mut comp_manager = ComponentManager::new();
        
        // Create a simple processor module for testing
        let proc_module = ProcessorModule::new(
            "test_processor",
            vec![PortSpec::input("in")],
            vec![PortSpec::output("out")],
            vec![],
            |_ctx, _outputs| Ok(()),
        );
        
        // Register the module
        comp_manager.register_module("test_processor", ComponentModule::Processing(proc_module)).unwrap();
        
        // Create a component instance
        let instance = comp_manager.create_component("test_processor", "test_comp_1".to_string()).unwrap();
        
        // Register component with engine
        engine.register_component(instance).unwrap();
        
        // Build execution order - this should use ExecutionOrderBuilder now
        let result = engine.build_execution_order();
        
        assert!(result.is_ok(), "Execution order building should succeed");
        
        // Verify execution order contains our component
        let order = engine.execution_order();
        assert_eq!(order.len(), 1);
        assert_eq!(order[0].id(), "test_comp_1");
    }

    #[test]
    fn test_cycle_detection() {
        // Test that cycle detection works through ExecutionOrderBuilder
        let mut engine = CycleEngine::new();
        
        // Create component manager and modules
        let mut comp_manager = ComponentManager::new();
        
        let proc_module = ProcessorModule::new(
            "test_processor",
            vec![PortSpec::input("in")],
            vec![PortSpec::output("out")],
            vec![],
            |_ctx, _outputs| Ok(()),
        );
        
        comp_manager.register_module("test_processor", ComponentModule::Processing(proc_module)).unwrap();
        
        // Create two components
        let instance1 = comp_manager.create_component("test_processor", "comp1".to_string()).unwrap();
        let instance2 = comp_manager.create_component("test_processor", "comp2".to_string()).unwrap();
        
        engine.register_component(instance1).unwrap();
        engine.register_component(instance2).unwrap();
        
        // Create a cycle: comp1 -> comp2 -> comp1
        let comp1_id = ComponentId::new("comp1".to_string(), "test_processor".to_string());
        let comp2_id = ComponentId::new("comp2".to_string(), "test_processor".to_string());
        engine.connect((comp1_id.clone(), "out".to_string()), (comp2_id.clone(), "in".to_string())).unwrap();
        engine.connect((comp2_id, "out".to_string()), (comp1_id, "in".to_string())).unwrap();
        
        // Building execution order should detect the cycle
        let result = engine.build_execution_order();
        assert!(result.is_err(), "Should detect cycle in component dependencies");
        assert!(result.unwrap_err().contains("Cycle detected"));
    }

    #[test]
    fn test_placeholder() {
        // Keep the placeholder for compatibility
        assert!(true);
    }
}