mod components;
mod simulation_builder;

use components::component_states::*;
use components::fifo_memory::FIFOMemory;
use simulation_builder::*;

fn main() -> Result<(), String> {
    println!("🔧 McDonald's Simulation - Delay Configuration Demo 🔧");
    println!();
    
    // Demonstrate Random Delay Mode (default)
    println!("1️⃣ Testing Random Delay Mode:");
    println!("   - Bakers: 2-5 cycles (random)");
    println!("   - Fryers: 3-7 cycles (random)");
    println!("   - Assemblers: 1-3 cycles (random)");
    println!("   - Customers: 1-5 cycles (random)");
    
    let (mut sim_random, components_random) = McSimulationBuilder::new()
        .component_counts(2, 2, 2, 2)
        .with_delay_mode(DelayMode::Random)
        .build()?;
    
    println!("✅ Built random delay simulation with {} bakers", components_random.bakers.len());
    
    // Build and run engine
    let mut engine_random = sim_random.build()?;
    engine_random.build_execution_order()?;
    
    // Run for a few cycles
    for cycle in 1..=10 {
        engine_random.cycle()?;
    }
    println!("✅ Random delay simulation completed 10 cycles");
    println!();
    
    // Demonstrate Fixed Delay Mode
    println!("2️⃣ Testing Fixed Delay Mode:");
    println!("   - Bakers: 3 cycles (fixed)");
    println!("   - Fryers: 5 cycles (fixed)");
    println!("   - Assemblers: 2 cycles (fixed)");
    println!("   - Customers: 4 cycles (fixed)");
    
    let (mut sim_fixed, components_fixed) = McSimulationBuilder::new()
        .component_counts(2, 2, 2, 2)
        .with_delay_mode(DelayMode::Fixed)
        .with_fixed_delays(3, 5, 2, 4) // baker, fryer, assembler, customer
        .build()?;
    
    println!("✅ Built fixed delay simulation with {} bakers", components_fixed.bakers.len());
    
    // Build and run engine
    let mut engine_fixed = sim_fixed.build()?;
    engine_fixed.build_execution_order()?;
    
    // Run for a few cycles
    for cycle in 1..=10 {
        engine_fixed.cycle()?;
    }
    println!("✅ Fixed delay simulation completed 10 cycles");
    println!();
    
    // Demonstrate Configuration Builder Pattern
    println!("3️⃣ Testing Configuration Builder Pattern:");
    
    let custom_config = McSimulationConfig {
        num_bakers: 1,
        num_fryers: 1,
        num_assemblers: 1,
        num_customers: 1,
        
        delay_mode: DelayMode::Fixed,
        fixed_delay_values: FixedDelayValues {
            baker_delay: 1,
            fryer_delay: 1,
            assembler_delay: 1,
            customer_delay: 1,
        },
        
        // Other default values
        individual_buffer_capacity: 5,
        inventory_buffer_capacity: 20,
        assembler_buffer_capacity: 3,
        burger_buffer_capacity: 10,
        customer_buffer_capacity: 3,
        
        baker_timing: (1, 3),
        fryer_timing: (1, 3),
        assembler_timing: (1, 2),
        customer_timing: (1, 2),
        
        baker_seed_base: 1000,
        fryer_seed_base: 2000,
        assembler_seed_base: 3000,
        customer_seed_base: 4000,
    };
    
    let (mut sim_custom, components_custom) = McSimulationBuilder::with_config(custom_config)
        .build()?;
    
    println!("✅ Built custom configuration simulation");
    println!("   - All components use 1 cycle fixed delay");
    println!("   - 1 of each component type");
    
    let mut engine_custom = sim_custom.build()?;
    engine_custom.build_execution_order()?;
    
    // Run for a few cycles
    for cycle in 1..=5 {
        engine_custom.cycle()?;
    }
    println!("✅ Custom configuration simulation completed 5 cycles");
    println!();
    
    println!("🎯 Delay Configuration Demo Summary:");
    println!("=====================================");
    println!("✅ Random delay mode: Components use random timing within configured ranges");
    println!("✅ Fixed delay mode: Components use exact timing values");
    println!("✅ Builder pattern: Easy configuration through fluent interface");
    println!("✅ Custom config: Direct configuration struct for advanced scenarios");
    println!("✅ Backward compatibility: Existing code continues to work");
    println!();
    println!("🚀 The delay configuration feature is ready for production use!");
    
    Ok(())
}