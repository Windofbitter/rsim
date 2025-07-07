mod components;
mod simulation_builder;

use components::component_states::*;
use simulation_builder::*;
use rsim::core::execution::config::{ConcurrencyMode, SimulationConfig};

fn main() -> Result<(), String> {
    println!("🔍 Checking execution order");
    
    let test_config = McSimulationConfig {
        num_bakers: 1,
        num_fryers: 1,
        num_assemblers: 1,
        num_customers: 1,
        individual_buffer_capacity: 5,
        inventory_buffer_capacity: 20,
        assembler_buffer_capacity: 3,
        burger_buffer_capacity: 10,
        customer_buffer_capacity: 5,
        delay_mode: DelayMode::Fixed,
        fixed_delay_values: FixedDelayValues {
            baker_delay: 2,
            fryer_delay: 3,
            assembler_delay: 1,
            customer_delay: 2,
        },
        baker_timing: (2, 5),
        fryer_timing: (3, 7),
        assembler_timing: (1, 3),
        customer_timing: (1, 5),
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
    
    println!("Components:");
    println!("  Assembler: {:?}", components.assemblers[0]);
    println!("  Customer: {:?}", components.customers[0]);
    println!("  Burger buffer: {:?}", components.burger_buffer);
    
    Ok(())
}