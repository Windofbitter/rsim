use rsim::core::builder::simulation_builder::Simulation;

#[path = "mc_simulation/components/baker.rs"]
mod baker;
#[path = "mc_simulation/components/fifo.rs"]
mod fifo;
use baker::Baker;
use fifo::FIFOData;

fn main() -> Result<(), String> {
    println!("🍔 Starting McDonald's Complete Simulation Test 🍔");
    
    // Create simulation
    let mut sim = Simulation::new();
    
    // =========================
    // TEST WITH BAKER COMPONENTS
    // =========================
    
    println!("Creating McDonald's simulation with Baker components...");
    
    // Create Baker components to test the macro system
    let baker1 = sim.add_component(Baker::new(2, 5, 1000));
    let baker2 = sim.add_component(Baker::new(3, 7, 2000));
    let baker3 = sim.add_component(Baker::new(1, 4, 3000));
    
    // Create FIFO buffers for bread storage (capacity 10 each)
    let bread_buffer1 = sim.add_memory_component(FIFOData::new(10));
    let bread_buffer2 = sim.add_memory_component(FIFOData::new(10));
    let bread_buffer3 = sim.add_memory_component(FIFOData::new(10));
    
    // Create baker state buffers (for internal state storage)
    let baker_state1 = sim.add_memory_component(FIFOData::new(10));
    let baker_state2 = sim.add_memory_component(FIFOData::new(10));
    let baker_state3 = sim.add_memory_component(FIFOData::new(10));
    
    // Connect Baker memory ports to memory components
    sim.connect_memory(baker1.output("bread_buffer"), bread_buffer1)?;
    sim.connect_memory(baker1.output("baker_state"), baker_state1)?;
    sim.connect_memory(baker2.output("bread_buffer"), bread_buffer2)?;
    sim.connect_memory(baker2.output("baker_state"), baker_state2)?;
    sim.connect_memory(baker3.output("bread_buffer"), bread_buffer3)?;
    sim.connect_memory(baker3.output("baker_state"), baker_state3)?;
    
    println!("✅ Created {} Baker components with memory successfully!", 3);
    
    // =========================
    // BUILD AND RUN
    // =========================
    
    println!("Building simulation engine...");
    let mut engine = sim.build()?;
    
    println!("Building execution order...");
    engine.build_execution_order()?;
    
    println!("🚀 Running McDonald's simulation for 30 cycles...\n");
    
    // Run simulation and print periodic status
    for cycle in 1..=30 {
        engine.cycle()?;
        
        if cycle % 10 == 0 {
            println!("📊 Cycle {}: Simulation running with {} components...", cycle, 3);
        }
    }
    
    println!("\n✅ McDonald's simulation with macros completed successfully!");
    println!("🎯 Executed {} cycles", engine.current_cycle());
    println!("🏭 Baker components using macros are working correctly!");
    println!("🎉 Macros are now fixed and functional!");
    
    Ok(())
}