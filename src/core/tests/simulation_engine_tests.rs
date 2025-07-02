// Tests for SimulationEngine functionality
#[cfg(test)]
mod tests {
    use crate::core::{
        execution::cycle_engine::CycleEngine,
        execution::simulation_engine::SimulationEngine,
        components::manager::ComponentManager,
        components::module::{ComponentModule, ProcessorModule, PortSpec},
    };

    #[test]
    fn test_step_return_type() {
        // Test that step() returns Result<(), String> instead of meaningless boolean
        let mut comp_manager = ComponentManager::new();
        
        // Create a simple processor module for testing
        let proc_module = ProcessorModule::new(
            "test_processor",
            vec![],  // No input ports for simple test
            vec![PortSpec::output("out")],
            vec![],
            |_ctx, _outputs| Ok(()),
        );
        
        comp_manager.register_module("test_processor", ComponentModule::Processing(proc_module)).unwrap();
        
        // Create a component instance
        let instance = comp_manager.create_component("test_processor", "test_comp".to_string()).unwrap();
        
        // Create cycle engine and register component
        let mut cycle_engine = CycleEngine::new();
        cycle_engine.register_component(instance).unwrap();
        
        // Create simulation engine
        let mut sim_engine = SimulationEngine::new(cycle_engine, Some(1)).unwrap();
        
        // Test that step() returns Result<(), String>
        let result = sim_engine.step();
        
        assert!(result.is_ok(), "Step should succeed");
        
        // Verify the return type by checking we can assign to () 
        let _unit_result: () = result.unwrap();
        
        // Verify cycle incremented
        assert_eq!(sim_engine.current_cycle(), 1);
    }

    #[test]
    fn test_run_with_max_cycles() {
        // Test that run() works correctly with the updated step() return type
        let mut comp_manager = ComponentManager::new();
        
        let proc_module = ProcessorModule::new(
            "test_processor",
            vec![],
            vec![PortSpec::output("out")],
            vec![],
            |_ctx, _outputs| Ok(()),
        );
        
        comp_manager.register_module("test_processor", ComponentModule::Processing(proc_module)).unwrap();
        let instance = comp_manager.create_component("test_processor", "test_comp".to_string()).unwrap();
        
        let mut cycle_engine = CycleEngine::new();
        cycle_engine.register_component(instance).unwrap();
        
        // Create simulation engine with max 3 cycles
        let mut sim_engine = SimulationEngine::new(cycle_engine, Some(3)).unwrap();
        
        // Run simulation
        let final_cycle = sim_engine.run().unwrap();
        
        assert_eq!(final_cycle, 3, "Should run exactly 3 cycles");
        assert_eq!(sim_engine.current_cycle(), 3, "Current cycle should be 3");
    }
}