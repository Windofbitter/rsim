// Integration tests for Phase 4 memory system thread safety implementation

#[cfg(test)]
mod tests {
    use crate::core::execution::config::{SimulationConfig, ConcurrencyMode};
    use crate::core::builder::simulation_builder::Simulation;
    use crate::core::execution::cycle_engine::CycleEngine;
    use crate::core::components::{Component, PortType, MemoryComponent, Cycle};
    use crate::core::components::module::ProcessorModule;
    use crate::core::components::module::PortSpec;
    use crate::core::components::state::MemoryData;
    use crate::core::values::traits::{EventInputs, EventOutputs};

    /// Test memory component for Phase 4 testing
    #[derive(Clone, Debug)]
    struct TestMemory {
        data: i32,
    }

    impl MemoryData for TestMemory {}

    impl Cycle for TestMemory {
        type Output = i32;
        
        fn cycle(&mut self) -> Option<Self::Output> {
            Some(self.data)
        }
    }

    impl MemoryComponent for TestMemory {
        fn define_ports() -> Vec<(String, PortType)> {
            vec![
                ("input".to_string(), PortType::Input),
                ("output".to_string(), PortType::Output),
            ]
        }
    }

    /// Test processor that uses memory
    struct TestProcessor;

    impl Component for TestProcessor {
        fn define_ports() -> Vec<(String, PortType)> {
            vec![
                ("input".to_string(), PortType::Input),
                ("output".to_string(), PortType::Output),
                ("memory".to_string(), PortType::Memory),
            ]
        }
        
        fn into_module() -> ProcessorModule {
            let ports = Self::define_ports();
            let input_ports = ports.iter().filter(|(_, t)| *t == PortType::Input)
                .map(|(name, _)| PortSpec::input(name)).collect();
            let output_ports = ports.iter().filter(|(_, t)| *t == PortType::Output)
                .map(|(name, _)| PortSpec::output(name)).collect();
            let memory_ports = ports.iter().filter(|(_, t)| *t == PortType::Memory)
                .map(|(name, _)| PortSpec::memory(name)).collect();
            
            ProcessorModule::new(
                "TestProcessor", 
                input_ports, 
                output_ports, 
                memory_ports,
                |ctx, outputs| {
                    // Read from memory
                    if let Ok(Some(stored_value)) = ctx.memory.read::<TestMemory>("memory", "state") {
                        outputs.set("output", stored_value.data)?;
                    }
                    
                    // Write to memory
                    if let Ok(input_value) = ctx.inputs.get::<i32>("input") {
                        ctx.memory.write("memory", "state", TestMemory { data: input_value })?;
                    }
                    
                    Ok(())
                }
            )
        }
    }

    #[test]
    fn test_memory_subset_pre_computation() {
        let mut sim = Simulation::new();
        
        // Add processor and memory components
        let processor = sim.add_component(TestProcessor);
        let memory = sim.add_memory_component(TestMemory { data: 42 });
        
        // Connect processor to memory
        sim.connect_memory(processor.output("memory"), memory.clone()).unwrap();
        
        // Build engine
        let mut engine = sim.build().unwrap();
        
        // Build execution order (this triggers memory subset pre-computation)
        engine.build_execution_order().unwrap();
        
        // Test that we can run cycles (this uses the memory subset system)
        for _ in 0..3 {
            engine.cycle().unwrap();
        }
        
        assert_eq!(engine.current_cycle(), 3);
    }

    #[test]
    fn test_memory_subset_with_multiple_components() {
        let mut sim = Simulation::new();
        
        // Add multiple processors and memory components
        let processor1 = sim.add_component(TestProcessor);
        let processor2 = sim.add_component(TestProcessor);
        let memory1 = sim.add_memory_component(TestMemory { data: 10 });
        let memory2 = sim.add_memory_component(TestMemory { data: 20 });
        
        // Connect processors to different memories
        sim.connect_memory(processor1.output("memory"), memory1.clone()).unwrap();
        sim.connect_memory(processor2.output("memory"), memory2.clone()).unwrap();
        
        // Build engine
        let mut engine = sim.build().unwrap();
        
        // Build execution order (this triggers memory subset pre-computation)
        engine.build_execution_order().unwrap();
        
        // Test that we can run cycles with multiple memory components
        for _ in 0..3 {
            engine.cycle().unwrap();
        }
        
        assert_eq!(engine.current_cycle(), 3);
    }

    #[test]
    fn test_backward_compatibility_with_existing_memory_system() {
        // Test that existing memory system continues to work
        let mut sim = Simulation::new();
        
        let processor = sim.add_component(TestProcessor);
        let memory = sim.add_memory_component(TestMemory { data: 100 });
        
        sim.connect_memory(processor.output("memory"), memory.clone()).unwrap();
        
        let mut engine = sim.build().unwrap();
        engine.build_execution_order().unwrap();
        
        // Should work exactly like before
        engine.cycle().unwrap();
        assert_eq!(engine.current_cycle(), 1);
    }

    #[test]
    fn test_sequential_mode_still_works() {
        // Test that sequential mode continues to work with the new memory system
        let config = SimulationConfig::new()
            .with_concurrency(ConcurrencyMode::Sequential);
        
        let mut sim = Simulation::with_config(config);
        
        let processor = sim.add_component(TestProcessor);
        let memory = sim.add_memory_component(TestMemory { data: 200 });
        
        sim.connect_memory(processor.output("memory"), memory.clone()).unwrap();
        
        let mut engine = sim.build().unwrap();
        engine.build_execution_order().unwrap();
        
        // Should work in sequential mode
        for _ in 0..5 {
            engine.cycle().unwrap();
        }
        
        assert_eq!(engine.current_cycle(), 5);
    }
}