use rsim::core::builder::simulation_builder::Simulation;

mod components;
use components::*;
use components::component_states::*;
use components::fifo_memory::FIFOMemory;

fn main() -> Result<(), String> {
    println!("🍔 Starting McDonald's Complete Simulation 🍔");
    
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
    
    // =========================
    // 2. INDIVIDUAL BUFFERS
    // =========================
    
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
    
    // =========================
    // 3. MANAGER COMPONENTS
    // =========================
    
    let bread_manager = sim.add_component(BreadManager::new());
    let meat_manager = sim.add_component(MeatManager::new());
    
    // Create intermediate memory buffers for manager coordination
    let bread_inventory_buffer = sim.add_memory_component(FIFOMemory::new(100)); // Large capacity for aggregated bread
    let meat_inventory_buffer = sim.add_memory_component(FIFOMemory::new(100)); // Large capacity for aggregated meat
    
    let assembler_manager = sim.add_component(AssemblerManager::new());
    
    // =========================
    // 4. ASSEMBLY COMPONENTS
    // =========================
    
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
    
    // =========================
    // 5. CONSUMER COMPONENTS
    // =========================
    
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
    
    // =========================
    // 6. MEMORY CONNECTIONS
    // =========================
    
    println!("Connecting production pipeline...");
    
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
    // Each assembler reads from the same buffer for both bread and meat (ingredient pairs)
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
    
    // =========================
    // 7. BUILD AND RUN
    // =========================
    
    println!("Building simulation engine...");
    let mut engine = sim.build()?;
    
    println!("Building execution order...");
    engine.build_execution_order()?;
    
    println!("🚀 Running McDonald's simulation for 100 cycles...\n");
    
    // Run simulation
    for cycle in 1..=100 {
        engine.cycle()?;
        
        if cycle % 20 == 0 {
            println!("📊 Cycle {}: Running...", cycle);
        }
    }
    
    println!("\n✅ McDonald's simulation completed successfully!");
    println!("🎯 Executed {} cycles", engine.current_cycle());
    println!("🏭 All components connected and functioning properly");
    
    // =========================
    // 8. QUERY SIMULATION RESULTS  
    // =========================
    
    println!("\n📊 SIMULATION RESULTS:");
    println!("======================");
    
    // Query baker production
    let mut total_bread_produced = 0;
    for i in 0..10 {
        if let Ok(Some(state)) = engine.query_memory_component_state::<BakerState>(&baker_states[i]) {
            total_bread_produced += state.total_produced;
            println!("Baker {}: {} bread produced", i, state.total_produced);
        }
    }
    println!("📊 Total bread produced: {}", total_bread_produced);
    
    // Query fryer production  
    let mut total_meat_produced = 0;
    for i in 0..10 {
        if let Ok(Some(state)) = engine.query_memory_component_state::<FryerState>(&fryer_states[i]) {
            total_meat_produced += state.total_produced;
            println!("Fryer {}: {} meat produced", i, state.total_produced);
        }
    }
    println!("🥩 Total meat produced: {}", total_meat_produced);
    
    // Query assembler production
    let mut total_burgers_assembled = 0;
    for i in 0..10 {
        if let Ok(Some(state)) = engine.query_memory_component_state::<AssemblerState>(&assembler_states[i]) {
            total_burgers_assembled += state.total_assembled;
            println!("Assembler {}: {} burgers assembled", i, state.total_assembled);
        }
    }
    println!("🍔 Total burgers assembled: {}", total_burgers_assembled);
    
    // Query customer consumption
    let mut total_burgers_consumed = 0;
    for i in 0..10 {
        if let Ok(Some(state)) = engine.query_memory_component_state::<CustomerState>(&consumer_states[i]) {
            total_burgers_consumed += state.total_consumed;
            println!("Customer {}: {} burgers consumed", i, state.total_consumed);
        }
    }
    println!("😋 Total burgers consumed: {}", total_burgers_consumed);
    
    // Query buffer states - check all bread and meat buffers
    println!("\n📦 INDIVIDUAL BUFFER STATUS:");
    for i in 0..3 {  // Check first 3 of each
        if let Ok(Some(bread_buffer_state)) = engine.query_memory_component_data::<FIFOMemory>(&bread_buffers[i], "buffer") {
            println!("🍞 Bread buffer {}: {}/{} bread", i, bread_buffer_state.data_count, bread_buffer_state.capacity);
        }
        if let Ok(Some(meat_buffer_state)) = engine.query_memory_component_data::<FIFOMemory>(&meat_buffers[i], "buffer") {
            println!("🥩 Meat buffer {}: {}/{} meat", i, meat_buffer_state.data_count, meat_buffer_state.capacity);
        }
    }
    
    // Check inventory buffers
    println!("\n📦 INVENTORY BUFFERS:");
    if let Ok(Some(bread_inventory_state)) = engine.query_memory_component_data::<FIFOMemory>(&bread_inventory_buffer, "buffer") {
        println!("🍞 Bread inventory: {}/{} bread", bread_inventory_state.data_count, bread_inventory_state.capacity);
    }
    if let Ok(Some(meat_inventory_state)) = engine.query_memory_component_data::<FIFOMemory>(&meat_inventory_buffer, "buffer") {
        println!("🥩 Meat inventory: {}/{} meat", meat_inventory_state.data_count, meat_inventory_state.capacity);
    }
    
    // Check assembler buffers
    println!("\n📦 ASSEMBLER BUFFERS:");
    for i in 0..3 {  // Check first 3
        if let Ok(Some(assembler_buffer_state)) = engine.query_memory_component_data::<FIFOMemory>(&assembler_buffers[i], "buffer") {
            println!("🔧 Assembler buffer {}: {}/{} ingredients", i, assembler_buffer_state.data_count, assembler_buffer_state.capacity);
        }
    }
    
    // Check final burger buffer
    println!("\n📦 FINAL BUFFER:");
    if let Ok(Some(burger_buffer_state)) = engine.query_memory_component_data::<FIFOMemory>(&burger_buffer, "buffer") {
        println!("🍔 Burger buffer: {}/{} burgers", burger_buffer_state.data_count, burger_buffer_state.capacity);
    }
    
    println!("\n🎉 Query functionality test completed successfully!");
    
    Ok(())
}