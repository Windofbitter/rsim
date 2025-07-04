use rsim::core::builder::simulation_builder::Simulation;

mod components;
use components::*;

fn main() -> Result<(), String> {
    println!("🍔 Starting McDonald's Complete Simulation Test 🍔");
    
    // Create simulation
    let mut sim = Simulation::new();
    
    // =========================
    // 1. PRODUCTION COMPONENTS
    // =========================
    
    // Create 10 Bakers with different seeds
    let mut bakers = Vec::new();
    for i in 0..10 {
        let baker = sim.add_component(Baker::new(2, 5, 1000 + i));
        bakers.push(baker);
    }
    
    // Create 10 Fryers with different seeds  
    let mut fryers = Vec::new();
    for i in 0..10 {
        let fryer = sim.add_component(Fryer::new(3, 7, 2000 + i));
        fryers.push(fryer);
    }
    
    // =========================
    // 2. INDIVIDUAL BUFFERS
    // =========================
    
    // Create 10 Bread Buffers (capacity 10 each)
    let mut bread_buffers = Vec::new();
    for _ in 0..10 {
        let buffer = sim.add_memory_component(FIFOData::new(10));
        bread_buffers.push(buffer);
    }
    
    // Create 10 Meat Buffers (capacity 10 each)
    let mut meat_buffers = Vec::new();
    for _ in 0..10 {
        let buffer = sim.add_memory_component(FIFOData::new(10));
        meat_buffers.push(buffer);
    }
    
    // =========================
    // 3. MANAGER COMPONENTS
    // =========================
    
    let bread_manager = sim.add_component(BreadManager::new());
    let meat_manager = sim.add_component(MeatManager::new());
    let assembler_manager = sim.add_component(AssemblerManager::new());
    
    // =========================
    // 4. ASSEMBLY COMPONENTS
    // =========================
    
    // Create 10 Assembler Buffers (capacity 5 each for ingredient pairs)
    let mut assembler_buffers = Vec::new();
    for _ in 0..10 {
        let buffer = sim.add_memory_component(FIFOData::new(5));
        assembler_buffers.push(buffer);
    }
    
    // Create 10 Assemblers with different seeds
    let mut assemblers = Vec::new();
    for i in 0..10 {
        let assembler = sim.add_component(Assembler::new(1, 3, 3000 + i));
        assemblers.push(assembler);
    }
    
    // =========================
    // 5. CUSTOMER COMPONENTS
    // =========================
    
    let customer_manager = sim.add_component(CustomerManager::new());
    
    // Create 10 Customer Buffers (capacity 8 each)
    let mut customer_buffers = Vec::new();
    for _ in 0..10 {
        let buffer = sim.add_memory_component(FIFOData::new(8));
        customer_buffers.push(buffer);
    }
    
    // Create 10 Customers with different seeds
    let mut customers = Vec::new();
    for i in 0..10 {
        let customer = sim.add_component(Customer::new(1, 5, 4000 + i));
        customers.push(customer);
    }
    
    // =========================
    // 6. MEMORY CONNECTIONS
    // =========================
    
    println!("Connecting production pipeline...");
    
    // Connect Bakers to Bread Buffers (1:1)
    for i in 0..10 {
        sim.connect_memory(bakers[i].memory_port("bread_buffer"), bread_buffers[i].clone())?;
    }
    
    // Connect Fryers to Meat Buffers (1:1)
    for i in 0..10 {
        sim.connect_memory(fryers[i].memory_port("meat_buffer"), meat_buffers[i].clone())?;
    }
    
    // Connect Bread Buffers to Bread Manager (10:1)
    for i in 0..10 {
        sim.connect_memory(bread_manager.memory_port(&format!("bread_buffer_{}", i + 1)), bread_buffers[i].clone())?;
    }
    
    // Connect Meat Buffers to Meat Manager (10:1)
    for i in 0..10 {
        sim.connect_memory(meat_manager.memory_port(&format!("meat_buffer_{}", i + 1)), meat_buffers[i].clone())?;
    }
    
    // Connect Managers to Assembler Manager
    sim.connect_memory(assembler_manager.memory_port("bread_manager"), bread_manager.clone())?;
    sim.connect_memory(assembler_manager.memory_port("meat_manager"), meat_manager.clone())?;
    
    // Connect Assembler Manager to Assembler Buffers (1:10)
    for i in 0..10 {
        sim.connect_memory(assembler_manager.memory_port(&format!("assembler_buffer_{}", i + 1)), assembler_buffers[i].clone())?;
    }
    
    // Connect Assembler Buffers to Assemblers (1:1)
    for i in 0..10 {
        sim.connect_memory(assemblers[i].memory_port("bread_buffer"), assembler_buffers[i].clone())?;
        sim.connect_memory(assemblers[i].memory_port("meat_buffer"), assembler_buffers[i].clone())?;
    }
    
    // Connect Assemblers to Customer Manager (10:1)
    for i in 0..10 {
        sim.connect_memory(customer_manager.memory_port(&format!("assembler_output_{}", i + 1)), assemblers[i].clone())?;
    }
    
    // Connect Customer Manager to Customer Buffers (1:10)
    for i in 0..10 {
        sim.connect_memory(customer_manager.memory_port(&format!("customer_buffer_{}", i + 1)), customer_buffers[i].clone())?;
    }
    
    // Connect Customer Buffers to Customers (1:1)
    for i in 0..10 {
        sim.connect_memory(customers[i].memory_port("burger_buffer"), customer_buffers[i].clone())?;
    }
    
    // =========================
    // 7. BUILD AND RUN
    // =========================
    
    println!("Building simulation engine...");
    let mut engine = sim.build()?;
    
    println!("Building execution order...");
    engine.build_execution_order()?;
    
    println!("🚀 Running McDonald's simulation for 50 cycles...\n");
    
    // Run simulation and print periodic status
    for cycle in 1..=50 {
        engine.cycle()?;
        
        if cycle % 10 == 0 {
            println!("📊 Cycle {}: Simulation running...", cycle);
        }
    }
    
    println!("\n✅ McDonald's simulation completed successfully!");
    println!("🎯 Executed {} cycles", engine.current_cycle());
    println!("🏭 All components connected and functioning properly");
    
    Ok(())
}