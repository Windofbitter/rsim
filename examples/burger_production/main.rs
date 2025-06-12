// Include the burger production module directly
#[path = "mod.rs"]
mod burger_production;

use rsim::core::{
    simulation_engine::SimulationEngine,
    event_manager::EventManager,
    event_scheduler::EventScheduler,
    event::Event,
    types::ComponentValue,
};
use burger_production::{
    BurgerSimulationConfig,
    config::ProductionMode,
    Fryer, Baker, Assembler, Client,
    FriedMeatBuffer, CookedBreadBuffer, AssemblyBuffer,
    TriggerProductionEvent, GenerateOrderEvent,
};

fn main() {
    // Create configuration
    let config = BurgerSimulationConfig::new()
        .with_production_mode(ProductionMode::BufferBased)
        .with_simulation_duration(500)
        .with_buffer_capacities(5)
        .with_order_quantity_range(1, 3)
        .with_order_interval(20)
        .with_random_seed(Some(42))
        .with_metrics_enabled(true)
        .with_event_logging_enabled(true);

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Invalid configuration: {}", e);
        return;
    }

    println!("🍔 Burger Production Simulation");
    println!("================================");
    println!("Mode: {:?}", config.production_mode);
    println!("Duration: {} cycles", config.simulation_duration_cycles);
    println!("Buffer Capacities: {}", config.buffer_capacities.fried_meat_capacity);
    println!("Order Interval: {} cycles", config.order_generation.order_interval_cycles);
    println!("Order Quantity: {}-{}", 
        config.order_generation.min_quantity_per_order,
        config.order_generation.max_quantity_per_order
    );
    println!();

    // Create simulation engine with max cycles
    let mut engine = SimulationEngine::new(Some(config.simulation_duration_cycles));

    // Create buffers
    let fried_meat_buffer = FriedMeatBuffer::new(
        "fried_meat_buffer".to_string(),
        Some(config.buffer_capacities.fried_meat_capacity),
    );
    let cooked_bread_buffer = CookedBreadBuffer::new(
        "cooked_bread_buffer".to_string(),
        Some(config.buffer_capacities.cooked_bread_capacity),
    );
    let assembly_buffer = AssemblyBuffer::new(
        "assembly_buffer".to_string(),
        config.buffer_capacities.assembly_capacity,
    );

    // Create production components
    let fryer = Fryer::new(
        "fryer".to_string(),
        config.production_mode,
        config.processing_delays.frying_cycles,
    );
    let baker = Baker::new(
        "baker".to_string(),
        config.production_mode,
        config.processing_delays.baking_cycles,
    );
    let assembler = Assembler::new(
        "assembler".to_string(),
        config.production_mode,
        config.processing_delays.assembly_cycles,
    );

    // Create client
    let client = Client::new(
        "client".to_string(),
        config.production_mode,
        config.order_generation.order_interval_cycles,
        config.order_generation.min_quantity_per_order,
        config.order_generation.max_quantity_per_order,
        "assembly_buffer".to_string(),
        config.random_seed.unwrap_or(42),
    );

    // Register all components
    engine.register_component(Box::new(fryer)).unwrap();
    engine.register_component(Box::new(baker)).unwrap();
    engine.register_component(Box::new(fried_meat_buffer)).unwrap();
    engine.register_component(Box::new(cooked_bread_buffer)).unwrap();
    engine.register_component(Box::new(assembler)).unwrap();
    engine.register_component(Box::new(assembly_buffer)).unwrap();
    engine.register_component(Box::new(client)).unwrap();

    // Schedule initial events
    if config.production_mode == ProductionMode::BufferBased {
        // Start production immediately in buffer-based mode
        let fryer_trigger = TriggerProductionEvent::new(
            "system".to_string(),
            Some(vec!["fryer".to_string()]),
        );
        engine.schedule_initial_event(Box::new(fryer_trigger), 1);
        
        let baker_trigger = TriggerProductionEvent::new(
            "system".to_string(),
            Some(vec!["baker".to_string()]),
        );
        engine.schedule_initial_event(Box::new(baker_trigger), 1);
    }

    // Schedule first order generation
    let first_order = GenerateOrderEvent::new(
        "system".to_string(),
        Some(vec!["client".to_string()]),
    );
    engine.schedule_initial_event(Box::new(first_order), config.order_generation.order_interval_cycles);

    // Note: Event logging not available in this SimulationEngine version

    // Run simulation with detailed logging
    println!("Starting simulation...\n");
    let start_time = std::time::Instant::now();
    
    println!("Initial events scheduled:");
    println!("- Production triggers at cycle 1");
    println!("- First order generation at cycle {}", config.order_generation.order_interval_cycles);
    println!();
    
    // Run simulation step by step for first 20 cycles to see what happens
    let mut cycle_count = 0;
    println!("=== DETAILED SIMULATION TRACE ===");
    
    while engine.has_pending_events() && cycle_count < 150 {
        let current_cycle = engine.current_cycle();
        let has_events_before = engine.has_pending_events();
        
        println!("Before step {}: Cycle {}, Has events: {}", cycle_count + 1, current_cycle, has_events_before);
        
        if !engine.step().unwrap() {
            println!("Step returned false - no more events");
            break;
        }
        
        let new_cycle = engine.current_cycle();
        let has_events_after = engine.has_pending_events();
        
        println!("After step {}: Cycle {}, Has events: {}", cycle_count + 1, new_cycle, has_events_after);
        println!();
        
        cycle_count += 1;
        
        if new_cycle >= config.simulation_duration_cycles {
            println!("Reached max cycles: {}", config.simulation_duration_cycles);
            break;
        }
    }
    
    // Finish remaining simulation
    let final_cycle = if engine.has_pending_events() {
        println!("Continuing simulation to completion...");
        engine.run().unwrap()
    } else {
        engine.current_cycle()
    };
    
    let elapsed = start_time.elapsed();
    
    // Print results
    println!("\n================================");
    println!("Simulation Complete!");
    println!("================================");
    println!("Total cycles simulated: {}", final_cycle);
    println!("Real time elapsed: {:.2?}", elapsed);
    
    // Note: Component metrics not available in this SimulationEngine version
}