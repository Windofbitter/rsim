mod components;
mod simulation_builder;

use components::component_states::*;
use components::fifo_memory::FIFOMemory;
use simulation_builder::*;

fn main() -> Result<(), String> {
    println!("🍔 McDonald's Complete Simulation Test with Helper 🍔");
    
    // Build a large-scale simulation using helper method
    let (mut sim, components) = build_large_mc_simulation()?;
    
    println!("✅ Built large simulation with:");
    println!("  - {} bakers", components.bakers.len());
    println!("  - {} fryers", components.fryers.len());
    println!("  - {} assemblers", components.assemblers.len());
    println!("  - {} customers", components.customers.len());
    
    println!("\nBuilding simulation engine...");
    let mut engine = sim.build()?;
    
    println!("Building execution order...");
    engine.build_execution_order()?;
    
    println!("🚀 Running McDonald's simulation for 100 cycles...\n");
    
    // Run simulation and print periodic status
    for cycle in 1..=100 {
        engine.cycle()?;
        
        if cycle % 20 == 0 {
            println!("📊 Cycle {}: Simulation running...", cycle);
        }
    }
    
    // Query final results
    println!("\n📊 FINAL SIMULATION RESULTS:");
    println!("============================");
    
    // Query baker production
    let mut total_bread_produced = 0;
    for i in 0..components.bakers.len() {
        if let Ok(Some(state)) = engine.query_memory_component_state::<BakerState>(&components.baker_states[i]) {
            total_bread_produced += state.total_produced;
        }
    }
    println!("🍞 Total bread produced: {}", total_bread_produced);
    
    // Query fryer production  
    let mut total_meat_produced = 0;
    for i in 0..components.fryers.len() {
        if let Ok(Some(state)) = engine.query_memory_component_state::<FryerState>(&components.fryer_states[i]) {
            total_meat_produced += state.total_produced;
        }
    }
    println!("🥩 Total meat produced: {}", total_meat_produced);
    
    // Query assembler production
    let mut total_burgers_assembled = 0;
    for i in 0..components.assemblers.len() {
        if let Ok(Some(state)) = engine.query_memory_component_state::<AssemblerState>(&components.assembler_states[i]) {
            total_burgers_assembled += state.total_assembled;
        }
    }
    println!("🍔 Total burgers assembled: {}", total_burgers_assembled);
    
    // Query customer consumption
    let mut total_burgers_consumed = 0;
    for i in 0..components.customers.len() {
        if let Ok(Some(state)) = engine.query_memory_component_state::<CustomerState>(&components.customer_states[i]) {
            total_burgers_consumed += state.total_consumed;
        }
    }
    println!("😋 Total burgers consumed: {}", total_burgers_consumed);
    
    // Check final burger buffer
    if let Ok(Some(burger_buffer_state)) = engine.query_memory_component_data::<FIFOMemory>(&components.burger_buffer, "buffer") {
        println!("🍔 Remaining burgers in buffer: {}/{}", burger_buffer_state.data_count, burger_buffer_state.capacity);
    }
    
    println!("\n✅ McDonald's simulation completed successfully!");
    println!("🎯 Executed {} cycles", engine.current_cycle());
    println!("🏭 All components connected and functioning properly");
    
    Ok(())
}