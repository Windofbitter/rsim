use rsim::core::builder::simulation_builder::Simulation;

fn main() -> Result<(), String> {
    println!("🍔 Testing McDonald's Simulation Structure 🍔");
    
    // Create a simple simulation to test basic structure
    let sim = Simulation::new();
    
    println!("✅ Simulation created successfully!");
    
    // Build engine to test basic functionality
    let mut engine = sim.build()?;
    engine.build_execution_order()?;
    
    // Run a few cycles
    for cycle in 1..=5 {
        engine.cycle()?;
        println!("📊 Cycle {}: Empty simulation running...", cycle);
    }
    
    println!("\n✅ Basic simulation structure test completed!");
    println!("🎯 Executed {} cycles", engine.current_cycle());
    
    Ok(())
}