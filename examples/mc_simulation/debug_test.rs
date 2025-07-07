mod components;
mod simulation_builder;

use components::component_states::*;
use components::fifo_memory::FIFOMemory;
use simulation_builder::*;
use rsim::core::execution::config::{ConcurrencyMode, SimulationConfig};

fn main() -> Result<(), String> {
    println!("🔍 DEBUG TEST: Running 10 cycles with detailed logging");
    println!("======================================================");
    
    // Create a simple configuration
    let test_config = McSimulationConfig {
        // Just 1 of each component for simplicity
        num_bakers: 1,
        num_fryers: 1,
        num_assemblers: 1,
        num_customers: 1,
        
        // Small buffers
        individual_buffer_capacity: 5,
        inventory_buffer_capacity: 20,
        assembler_buffer_capacity: 3,
        burger_buffer_capacity: 10,
        customer_buffer_capacity: 5,
        
        // Fixed delays for deterministic behavior
        delay_mode: DelayMode::Fixed,
        fixed_delay_values: FixedDelayValues {
            baker_delay: 2,
            fryer_delay: 3,
            assembler_delay: 1,
            customer_delay: 2,
        },
        
        // Fallback values (not used in fixed mode)
        baker_timing: (2, 5),
        fryer_timing: (3, 7),
        assembler_timing: (1, 3),
        customer_timing: (1, 5),
        
        // Deterministic seeds
        baker_seed_base: 1000,
        fryer_seed_base: 2000,
        assembler_seed_base: 3000,
        customer_seed_base: 4000,
    };
    
    let sequential_config = SimulationConfig::new()
        .with_concurrency(ConcurrencyMode::Sequential);
    
    let (mut sim, components) = McSimulationBuilder::with_config(test_config)
        .build_with_config(sequential_config)?;
    
    let mut engine = sim.build()?;
    engine.build_execution_order()?;
    
    // Run longer to see if eventual consumption happens
    for cycle in 1..=50 {
        engine.cycle()?;
        
        // Query state after each cycle
        if let Ok(Some(customer_state)) = engine.query_memory_component_state::<CustomerState>(&components.customer_states[0]) {
            if let Ok(Some(burger_buffer_state)) = engine.query_memory_component_data::<FIFOMemory>(&components.burger_buffer, "buffer") {
                if let Ok(Some(assembler_state)) = engine.query_memory_component_state::<AssemblerState>(&components.assembler_states[0]) {
                    if cycle % 5 == 0 || customer_state.total_consumed > 0 || burger_buffer_state.data_count > 0 {
                        println!("CYCLE {}: consumed={}, buffer_count={}, assembled={}", 
                                cycle, customer_state.total_consumed, burger_buffer_state.data_count, assembler_state.total_assembled);
                    }
                }
            }
        }
    }
    
    Ok(())
}