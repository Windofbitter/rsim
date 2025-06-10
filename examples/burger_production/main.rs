use rsim::core::simulation_engine::SimulationEngine;
use uuid::Uuid;

mod events;
mod components;

use events::*;
use components::*;

/// Configuration for the burger production simulation
#[derive(Debug, Clone)]
pub struct BurgerSimulationConfig {
    // Processing delays (simulation time units)
    pub frying_delay: u64,
    pub baking_delay: u64,
    pub assembly_delay: u64,
    
    // Buffer capacities  
    pub meat_buffer_capacity: u32,
    pub bread_buffer_capacity: u32,
    pub assembly_buffer_capacity: u32,
    
    // Component concurrency (all set to 1 as requested)
    pub max_concurrent_items: u32,
    
    // Client behavior
    pub order_generation_interval: u64,
    pub order_size_mean: f64,
    pub order_size_std_dev: f64,
    
    // Simulation parameters
    pub max_simulation_cycles: u64,
    pub random_seed: u64,
}

impl Default for BurgerSimulationConfig {
    fn default() -> Self {
        Self {
            // Processing delays
            frying_delay: 10,
            baking_delay: 8,
            assembly_delay: 5,
            
            // Buffer capacities (all set to 5 as requested)
            meat_buffer_capacity: 5,
            bread_buffer_capacity: 5,
            assembly_buffer_capacity: 5,
            
            // Component concurrency (all set to 1 as requested)
            max_concurrent_items: 1,
            
            // Client behavior
            order_generation_interval: 15,
            order_size_mean: 2.0,
            order_size_std_dev: 0.5,
            
            // Simulation parameters
            max_simulation_cycles: 100,  // Reduced for clearer output
            random_seed: 42,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger with timestamps
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)  // Remove timestamps for cleaner output
        .init();
    
    println!("🍔 Starting Burger Production Simulation");
    
    let config = BurgerSimulationConfig::default();
    
    // Print configuration
    println!("Configuration:");
    println!("  Processing delays: frying={}, baking={}, assembly={}", 
             config.frying_delay, config.baking_delay, config.assembly_delay);
    println!("  Buffer capacities: meat={}, bread={}, assembly={}", 
             config.meat_buffer_capacity, config.bread_buffer_capacity, config.assembly_buffer_capacity);
    println!("  Max concurrent items: {}", config.max_concurrent_items);
    println!("  Order generation: interval={}, mean={:.1}, std_dev={:.1}", 
             config.order_generation_interval, config.order_size_mean, config.order_size_std_dev);
    println!("  Max simulation cycles: {}", config.max_simulation_cycles);
    println!();

    // Component IDs
    let fryer_id = "fryer".to_string();
    let baker_id = "baker".to_string();
    let assembler_id = "assembler".to_string();
    let meat_buffer_id = "fried_meat_buffer".to_string();
    let bread_buffer_id = "cooked_bread_buffer".to_string();
    let assembly_buffer_id = "assembly_buffer".to_string();
    let client_id = "client".to_string();
    let metrics_collector_id = "metrics_collector".to_string();

    // Create simulation engine
    let mut engine = SimulationEngine::new(Some(config.max_simulation_cycles));

    // Create FIFO buffers with subscribers
    let meat_buffer = FriedMeatBuffer::new(
        meat_buffer_id.clone(),
        config.meat_buffer_capacity,
        vec![assembler_id.clone()], // Assembler subscribes to meat buffer
    );
    
    let bread_buffer = CookedBreadBuffer::new(
        bread_buffer_id.clone(),
        config.bread_buffer_capacity,
        vec![assembler_id.clone()], // Assembler subscribes to bread buffer
    );
    
    let assembly_buffer = AssemblyBuffer::new(
        assembly_buffer_id.clone(),
        config.assembly_buffer_capacity,
        vec![client_id.clone()], // Client subscribes to assembly buffer
    );

    // Create production components
    let fryer = Fryer::new(
        fryer_id.clone(),
        meat_buffer_id.clone(),
        config.frying_delay,
        config.max_concurrent_items,
    );
    
    let baker = Baker::new(
        baker_id.clone(),
        bread_buffer_id.clone(),
        config.baking_delay,
        config.max_concurrent_items,
    );
    
    let assembler = Assembler::new(
        assembler_id.clone(),
        meat_buffer_id.clone(),
        bread_buffer_id.clone(),
        assembly_buffer_id.clone(),
        config.assembly_delay,
        config.max_concurrent_items,
    );

    // Create client component
    let client = Client::new(
        client_id.clone(),
        assembly_buffer_id.clone(),
        config.order_generation_interval,
        config.order_size_mean,
        config.order_size_std_dev,
        config.random_seed,
    );

    // Create metrics collector component
    let metrics_collector = MetricsCollector::new(
        metrics_collector_id.clone(),
        25, // Window size: 25 cycles
        25, // Report interval: every 25 cycles
    );

    // Register all components with simulation engine
    println!("Registering components...");
    engine.register_component(Box::new(fryer))?;
    engine.register_component(Box::new(baker))?;
    engine.register_component(Box::new(assembler))?;
    engine.register_component(Box::new(meat_buffer))?;
    engine.register_component(Box::new(bread_buffer))?;
    engine.register_component(Box::new(assembly_buffer))?;
    engine.register_component(Box::new(client))?;
    engine.register_component(Box::new(metrics_collector))?;

    // Create initial events to kickstart the simulation
    println!("Scheduling initial events...");
    
    // Start fryer
    let start_frying_event = Box::new(StartFryingEvent {
        id: Uuid::new_v4().to_string(),
        source_id: fryer_id.clone(),
    });
    engine.schedule_initial_event(start_frying_event, 1);

    // Start baker
    let start_baking_event = Box::new(StartBakingEvent {
        id: Uuid::new_v4().to_string(),
        source_id: baker_id.clone(),
    });
    engine.schedule_initial_event(start_baking_event, 1);

    // Start assembler (with placeholder IDs)
    let start_assembly_event = Box::new(StartAssemblyEvent {
        id: Uuid::new_v4().to_string(),
        source_id: assembler_id.clone(),
        meat_id: "initial".to_string(),
        bread_id: "initial".to_string(),
    });
    engine.schedule_initial_event(start_assembly_event, 5); // Start after some ingredients are ready

    // Start client order generation
    let generate_order_event = Box::new(GenerateOrderEvent {
        id: Uuid::new_v4().to_string(),
        source_id: client_id.clone(),
    });
    engine.schedule_initial_event(generate_order_event, 20); // Start orders after production begins

    // Start metrics reporting
    let initial_metrics_event = Box::new(MetricsReportEvent {
        id: Uuid::new_v4().to_string(),
        source_id: metrics_collector_id.clone(),
        report_time: 25,
    });
    engine.schedule_initial_event(initial_metrics_event, 25); // First report at cycle 25
    
    println!("Starting simulation for {} cycles...\n", config.max_simulation_cycles);
    
    let result = engine.run();
    
    match result {
        Ok(cycles_executed) => {
            println!("\n✅ Simulation completed successfully!");
            println!("Cycles executed: {}", cycles_executed);
            println!("Final statistics would be displayed here");
        }
        Err(e) => {
            println!("\n❌ Simulation failed: {:?}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}