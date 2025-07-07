mod components;
mod simulation_builder;

use components::component_states::*;
use simulation_builder::*;
use rsim::core::execution::config::{ConcurrencyMode, SimulationConfig};
use rsim::core::execution::execution_order::ExecutionOrderBuilder;

fn main() -> Result<(), String> {
    println!("🔍 McDonald's Simulation Execution Order Analysis");
    println!("================================================");
    
    // Create test configuration
    let config = McSimulationConfig {
        num_bakers: 2,
        num_fryers: 2,
        num_assemblers: 2,
        num_customers: 2,
        ..Default::default()
    };
    
    // Build simulation
    let (mut sim, components) = McSimulationBuilder::with_config(config)
        .build_with_config(SimulationConfig::default())?;
    
    // Build engine to analyze execution order
    let mut engine = sim.build()?;
    
    // Get all processing components
    let processing_components: Vec<_> = engine.component_ids().into_iter().cloned().collect();
    
    println!("\n📋 All Components:");
    for comp in &processing_components {
        println!("  - {}", comp);
    }
    
    // Build execution order first
    engine.build_execution_order()?;
    
    println!("\n📊 Current cycle: {}", engine.current_cycle());
    
    // We can't access internal fields, so let's create a test to understand dependencies
    // Create a simple test with known dependencies
    use rsim::core::execution::execution_order::ExecutionOrderBuilder;
    use std::collections::HashMap;
    use rsim::core::types::ComponentId;
    
    // Create dummy components to test topological sort
    let comp_a = ComponentId::new("CompA".to_string(), "test".to_string());
    let comp_b = ComponentId::new("CompB".to_string(), "test".to_string());
    let comp_c = ComponentId::new("CompC".to_string(), "test".to_string());
    let test_components = vec![comp_a.clone(), comp_b.clone(), comp_c.clone()];
    
    // Test 1: No connections (should be single stage with all components)
    let mut test_connections = HashMap::new();
    let stages = ExecutionOrderBuilder::build_execution_order_stages(&test_components, &test_connections)?;
    println!("\n🧪 Test 1 - No connections:");
    println!("  Stages: {:?}", stages);
    
    // Test 2: Linear dependencies A -> B -> C
    test_connections.insert(
        (comp_a.clone(), "out".to_string()),
        vec![(comp_b.clone(), "in".to_string())]
    );
    test_connections.insert(
        (comp_b.clone(), "out".to_string()),
        vec![(comp_c.clone(), "in".to_string())]
    );
    let stages = ExecutionOrderBuilder::build_execution_order_stages(&test_components, &test_connections)?;
    println!("\n🧪 Test 2 - Linear dependencies (A->B->C):");
    println!("  Stages: {:?}", stages);
    
    println!("\n🔍 ANALYSIS:");
    println!("The McDonald's simulation likely has NO PORT CONNECTIONS between processing components!");
    println!("All components probably connect only through memory components, which means");
    println!("the topological sort sees them as independent (no dependencies).");
    println!("This would result in all components being placed in a single stage.");
    println!("In parallel mode, this means customers could execute before assemblers");
    
    Ok(())
}