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
    // Initialize logger
    env_logger::init();
    
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
        log::error!("Invalid configuration: {}", e);
        return;
    }

    log::info!("🍔 Burger Production Simulation");
    log::info!("================================");
    log::info!("Mode: {:?}", config.production_mode);
    log::info!("Duration: {} cycles", config.simulation_duration_cycles);
    log::info!("Buffer Capacities: {}", config.buffer_capacities.fried_meat_capacity);
    log::info!("Order Interval: {} cycles", config.order_generation.order_interval_cycles);
    log::info!("Order Quantity: {}-{}", 
        config.order_generation.min_quantity_per_order,
        config.order_generation.max_quantity_per_order
    );

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
    log::info!("Starting simulation...");
    let start_time = std::time::Instant::now();
    
    log::info!("Initial events scheduled:");
    log::info!("- Production triggers at cycle 1");
    log::info!("- First order generation at cycle {}", config.order_generation.order_interval_cycles);
    
    // Run simulation step by step for first 20 cycles to see what happens
    let mut cycle_count = 0;
    log::debug!("=== DETAILED SIMULATION TRACE ===");
    
    while engine.has_pending_events() && cycle_count < 150 {
        let current_cycle = engine.current_cycle();
        let has_events_before = engine.has_pending_events();
        
        log::debug!("Before step {}: Cycle {}, Has events: {}", cycle_count + 1, current_cycle, has_events_before);
        
        if !engine.step().unwrap() {
            log::debug!("Step returned false - no more events");
            break;
        }
        
        let new_cycle = engine.current_cycle();
        let has_events_after = engine.has_pending_events();
        
        log::debug!("After step {}: Cycle {}, Has events: {}", cycle_count + 1, new_cycle, has_events_after);
        
        cycle_count += 1;
        
        if new_cycle >= config.simulation_duration_cycles {
            log::info!("Reached max cycles: {}", config.simulation_duration_cycles);
            break;
        }
    }
    
    // Finish remaining simulation
    let final_cycle = if engine.has_pending_events() {
        log::info!("Continuing simulation to completion...");
        engine.run().unwrap()
    } else {
        engine.current_cycle()
    };
    
    let elapsed = start_time.elapsed();
    
    // Print results
    log::info!("================================");
    log::info!("Simulation Complete!");
    log::info!("================================");
    log::info!("Total cycles simulated: {}", final_cycle);
    log::info!("Real time elapsed: {:.2?}", elapsed);
    
    // Note: Component metrics not available in this SimulationEngine version
}