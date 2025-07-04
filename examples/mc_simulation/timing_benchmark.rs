use rsim::core::builder::simulation_builder::Simulation;
use rsim::core::execution::cycle_engine::CycleEngine;
use std::time::Instant;

mod components;
use components::*;
use components::component_states::*;
use components::fifo_memory::FIFOMemory;

/// Builds a complete McDonald's simulation with all components
fn build_mcdonalds_simulation() -> Result<CycleEngine, String> {
    let mut sim = Simulation::new();
    
    // Create 10 Bakers with different seeds
    let mut bakers = Vec::new();
    for i in 0..10 {
        let baker = sim.add_component(Baker::new(2, 5, 1000 + i));
        bakers.push(baker);
    }
    
    // Create state memory components for each baker
    let mut baker_states = Vec::new();
    for _ in 0..10 {
        let state = sim.add_memory_component(BakerState::new());
        baker_states.push(state);
    }
    
    // Create 10 Fryers with different seeds  
    let mut fryers = Vec::new();
    for i in 0..10 {
        let fryer = sim.add_component(Fryer::new(3, 7, 2000 + i));
        fryers.push(fryer);
    }
    
    // Create state memory components for each fryer
    let mut fryer_states = Vec::new();
    for _ in 0..10 {
        let state = sim.add_memory_component(FryerState::new());
        fryer_states.push(state);
    }
    
    // Create 10 Bread Buffers (capacity 10 each)
    let mut bread_buffers = Vec::new();
    for _ in 0..10 {
        let buffer = sim.add_memory_component(FIFOMemory::new(10));
        bread_buffers.push(buffer);
    }
    
    // Create 10 Meat Buffers (capacity 10 each)
    let mut meat_buffers = Vec::new();
    for _ in 0..10 {
        let buffer = sim.add_memory_component(FIFOMemory::new(10));
        meat_buffers.push(buffer);
    }
    
    // Manager components
    let bread_manager = sim.add_component(BreadManager::new());
    let meat_manager = sim.add_component(MeatManager::new());
    
    // Create intermediate memory buffers for manager coordination
    let bread_inventory_buffer = sim.add_memory_component(FIFOMemory::new(100));
    let meat_inventory_buffer = sim.add_memory_component(FIFOMemory::new(100));
    
    let assembler_manager = sim.add_component(AssemblerManager::new());
    
    // Create 10 Assembler Buffers (capacity 5 each for ingredient pairs)
    let mut assembler_buffers = Vec::new();
    for _ in 0..10 {
        let buffer = sim.add_memory_component(FIFOMemory::new(5));
        assembler_buffers.push(buffer);
    }
    
    // Create 10 Assemblers with different seeds
    let mut assemblers = Vec::new();
    for i in 0..10 {
        let assembler = sim.add_component(Assembler::new(1, 3, 3000 + i));
        assemblers.push(assembler);
    }
    
    // Create state memory components for each assembler
    let mut assembler_states = Vec::new();
    for _ in 0..10 {
        let state = sim.add_memory_component(AssemblerState::new());
        assembler_states.push(state);
    }
    
    // Create single shared burger buffer
    let burger_buffer = sim.add_memory_component(FIFOMemory::new(50));
    
    // Create 10 Consumers with different seeds
    let mut consumers = Vec::new();
    for i in 0..10 {
        let consumer = sim.add_component(Customer::new(1, 5, 4000 + i));
        consumers.push(consumer);
    }
    
    // Create state memory components for each consumer
    let mut consumer_states = Vec::new();
    for _ in 0..10 {
        let state = sim.add_memory_component(CustomerState::new());
        consumer_states.push(state);
    }
    
    // Connect Bakers to Bread Buffers (1:1)
    for i in 0..10 {
        sim.connect_memory_port(bakers[i].memory_port("bread_buffer"), bread_buffers[i].clone())?;
    }
    
    // Connect Baker State Memory (1:1)
    for i in 0..10 {
        sim.connect_memory_port(bakers[i].memory_port("baker_state"), baker_states[i].clone())?;
    }
    
    // Connect Fryers to Meat Buffers (1:1)
    for i in 0..10 {
        sim.connect_memory_port(fryers[i].memory_port("meat_buffer"), meat_buffers[i].clone())?;
    }
    
    // Connect Fryer State Memory (1:1)
    for i in 0..10 {
        sim.connect_memory_port(fryers[i].memory_port("fryer_state"), fryer_states[i].clone())?;
    }
    
    // Connect Bread Buffers to Bread Manager (10:1)
    for i in 0..10 {
        sim.connect_memory_port(bread_manager.memory_port(&format!("bread_buffer_{}", i + 1)), bread_buffers[i].clone())?;
    }
    
    // Connect Meat Buffers to Meat Manager (10:1)
    for i in 0..10 {
        sim.connect_memory_port(meat_manager.memory_port(&format!("meat_buffer_{}", i + 1)), meat_buffers[i].clone())?;
    }
    
    // Connect Managers to their inventory output buffers
    sim.connect_memory_port(bread_manager.memory_port("bread_inventory_out"), bread_inventory_buffer.clone())?;
    sim.connect_memory_port(meat_manager.memory_port("meat_inventory_out"), meat_inventory_buffer.clone())?;
    
    // Connect Assembler Manager to the inventory buffers  
    sim.connect_memory_port(assembler_manager.memory_port("bread_inventory"), bread_inventory_buffer.clone())?;
    sim.connect_memory_port(assembler_manager.memory_port("meat_inventory"), meat_inventory_buffer.clone())?;
    
    // Connect Assembler Manager to Assembler Buffers (1:10)
    for i in 0..10 {
        sim.connect_memory_port(assembler_manager.memory_port(&format!("assembler_buffer_{}", i + 1)), assembler_buffers[i].clone())?;
    }
    
    // Connect Assembler Buffers to Assemblers (1:1 for ingredient pairs)
    for i in 0..10 {
        sim.connect_memory_port(assemblers[i].memory_port("bread_buffer"), assembler_buffers[i].clone())?;
        sim.connect_memory_port(assemblers[i].memory_port("meat_buffer"), assembler_buffers[i].clone())?;
    }
    
    // Connect Assemblers to Burger Buffer (10:1)
    for i in 0..10 {
        sim.connect_memory_port(assemblers[i].memory_port("burger_buffer"), burger_buffer.clone())?;
    }
    
    // Connect Assembler State Memory (1:1)
    for i in 0..10 {
        sim.connect_memory_port(assemblers[i].memory_port("assembler_state"), assembler_states[i].clone())?;
    }
    
    // Connect Burger Buffer to Consumers (1:10)
    for i in 0..10 {
        sim.connect_memory_port(consumers[i].memory_port("burger_buffer"), burger_buffer.clone())?;
    }
    
    // Connect Consumer State Memory (1:1)
    for i in 0..10 {
        sim.connect_memory_port(consumers[i].memory_port("customer_state"), consumer_states[i].clone())?;
    }
    
    // Build simulation engine
    let mut engine = sim.build()?;
    engine.build_execution_order()?;
    
    Ok(engine)
}

/// Times the execution of a simulation for a given number of cycles
fn time_simulation(cycles: usize) -> Result<std::time::Duration, String> {
    let mut engine = build_mcdonalds_simulation()?;
    
    let start_time = Instant::now();
    
    for _ in 1..=cycles {
        engine.cycle()?;
    }
    
    let duration = start_time.elapsed();
    Ok(duration)
}

fn main() -> Result<(), String> {
    println!("🍔 McDonald's Simulation Timing Benchmark 🍔");
    println!("==============================================");
    
    let test_cases = [100, 1000, 10000];
    
    for &cycles in &test_cases {
        println!("\n⏱️  Testing {} cycles...", cycles);
        
        match time_simulation(cycles) {
            Ok(duration) => {
                let millis = duration.as_millis();
                let micros = duration.as_micros();
                let cycles_per_second = if millis > 0 { 
                    (cycles as f64 / (millis as f64 / 1000.0)) as u64
                } else {
                    cycles as u64 * 1_000_000 / micros as u64
                };
                
                println!("✅ {} cycles completed in {} ms ({} μs)", cycles, millis, micros);
                println!("📊 Performance: ~{} cycles/second", cycles_per_second);
            }
            Err(e) => {
                println!("❌ Error running {} cycles: {}", cycles, e);
            }
        }
    }
    
    println!("\n🎯 Benchmark completed!");
    
    Ok(())
}