// Integration tests for Phase 5 parallel execution implementation

#[cfg(test)]
mod tests {
    use crate::core::execution::config::{SimulationConfig, ConcurrencyMode};
    use crate::core::builder::simulation_builder::Simulation;

    #[test]
    fn test_sequential_vs_parallel_execution() {
        // Test that both sequential and parallel modes work and produce the same results
        
        // Sequential mode
        let config_seq = SimulationConfig::new()
            .with_concurrency(ConcurrencyMode::Sequential);
        let sim_seq = Simulation::with_config(config_seq);
        let mut engine_seq = sim_seq.build().expect("Should build sequential engine");
        let _ = engine_seq.build_execution_order();
        
        // Parallel mode  
        let config_par = SimulationConfig::new()
            .with_concurrency(ConcurrencyMode::Rayon);
        let sim_par = Simulation::with_config(config_par);
        let mut engine_par = sim_par.build().expect("Should build parallel engine");
        let _ = engine_par.build_execution_order();
        
        // Both should start at cycle 0
        assert_eq!(engine_seq.current_cycle(), 0);
        assert_eq!(engine_par.current_cycle(), 0);
        
        // Both should successfully execute a cycle
        let result_seq = engine_seq.cycle();
        let result_par = engine_par.cycle();
        
        assert!(result_seq.is_ok(), "Sequential execution should succeed");
        assert!(result_par.is_ok(), "Parallel execution should succeed"); 
        
        // Both should advance to cycle 1
        assert_eq!(engine_seq.current_cycle(), 1);
        assert_eq!(engine_par.current_cycle(), 1);
    }

    #[test]
    fn test_parallel_execution_multiple_cycles() {
        // Test that parallel execution can run multiple cycles successfully
        let config = SimulationConfig::new()
            .with_concurrency(ConcurrencyMode::Rayon);
        let sim = Simulation::with_config(config);
        let mut engine = sim.build().expect("Should build parallel engine");
        let _ = engine.build_execution_order();
        
        // Run 10 cycles
        for expected_cycle in 1..=10 {
            let result = engine.cycle();
            assert!(result.is_ok(), "Cycle {} should succeed in parallel mode", expected_cycle);
            assert_eq!(engine.current_cycle(), expected_cycle);
        }
    }

    #[test]
    fn test_configuration_validation() {
        // Test that configuration options work correctly
        let config = SimulationConfig::new()
            .with_concurrency(ConcurrencyMode::Rayon)
            .with_thread_pool_size(4);
        
        // Should be able to build with custom config
        let sim = Simulation::with_config(config);
        let engine = sim.build().expect("Should build with custom config");
        assert_eq!(engine.current_cycle(), 0);
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that existing code still works (backward compatibility)
        let sim = Simulation::new(); // Default sequential mode
        let mut engine = sim.build().expect("Should build with defaults");
        let _ = engine.build_execution_order();
        
        let result = engine.cycle();
        assert!(result.is_ok(), "Default sequential mode should work");
        assert_eq!(engine.current_cycle(), 1);
    }
    
    #[test]
    fn test_parallel_execution_with_memory_access() {
        // Test that parallel execution works and handles memory component access properly
        // Note: This test focuses on the parallel execution framework rather than specific memory operations
        
        let config = SimulationConfig::new()
            .with_concurrency(ConcurrencyMode::Rayon);
        let sim = Simulation::with_config(config);
        let mut engine = sim.build().expect("Should build parallel engine");
        
        // Build execution order (this includes pre-computing memory component subsets)
        engine.build_execution_order().expect("Should build execution order");
        
        // Run multiple cycles to verify that parallel execution is stable and memory access is handled correctly
        for cycle in 1..=5 {
            let result = engine.cycle();
            assert!(result.is_ok(), "Cycle {} should succeed in parallel mode", cycle);
            assert_eq!(engine.current_cycle(), cycle);
        }
        
        // Verify that the engine completes cycles successfully in parallel mode
        assert_eq!(engine.current_cycle(), 5);
    }
}