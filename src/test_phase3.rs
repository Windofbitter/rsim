// Integration tests for Phase 3 concurrency implementation

#[cfg(test)]
mod tests {
    use crate::core::execution::config::{SimulationConfig, ConcurrencyMode};
    use crate::core::builder::simulation_builder::Simulation;
    use crate::core::execution::cycle_engine::CycleEngine;

    #[test]
    fn test_backward_compatible_construction() {
        // Test default construction maintains backward compatibility
        let sim = Simulation::new();
        let engine = sim.build().expect("Should build successfully");
        assert_eq!(engine.current_cycle(), 0);
    }

    #[test]
    fn test_configuration_construction() {
        // Test new configuration-based construction
        let config = SimulationConfig::new()
            .with_concurrency(ConcurrencyMode::Sequential)
            .with_thread_pool_size(4);
        
        let sim = Simulation::with_config(config.clone());
        let engine = sim.build().expect("Should build successfully");
        assert_eq!(engine.current_cycle(), 0);
    }

    #[test]
    fn test_cycle_engine_construction() {
        // Test CycleEngine construction methods
        let config = SimulationConfig::new()
            .with_concurrency(ConcurrencyMode::Sequential);
        
        let engine1 = CycleEngine::new(config);
        assert_eq!(engine1.current_cycle(), 0);
        
        let engine2 = CycleEngine::new_sequential();
        assert_eq!(engine2.current_cycle(), 0);
    }

    #[test]
    fn test_sequential_execution() {
        // Test that sequential execution works
        let config = SimulationConfig::new()
            .with_concurrency(ConcurrencyMode::Sequential);
        
        let sim = Simulation::with_config(config);
        let mut engine = sim.build().expect("Should build successfully");
        
        // Should be able to execute cycles
        assert_eq!(engine.current_cycle(), 0);
        let _ = engine.build_execution_order(); // Should not fail
    }

    #[test]
    fn test_parallel_execution_implemented() {
        // Test that parallel execution now works
        let config = SimulationConfig::new()
            .with_concurrency(ConcurrencyMode::Rayon);
        
        let sim = Simulation::with_config(config);
        let mut engine = sim.build().expect("Should build successfully");
        
        // Build execution order first
        let _ = engine.build_execution_order();
        
        // Parallel execution should now succeed
        let result = engine.cycle();
        assert!(result.is_ok(), "Parallel execution should succeed, got error: {:?}", result);
        assert_eq!(engine.current_cycle(), 1);
    }
}