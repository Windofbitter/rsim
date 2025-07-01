// Include the burger production module directly
#[path = "mod.rs"]
mod burger_production;

use burger_production::{
    config::ProductionMode, Assembler, AssemblyBuffer, Baker, BurgerSimulationConfig, Client,
    CookedBreadBuffer, FriedMeatBuffer, Fryer, GenerateOrderEvent, MetricsCollector,
    TriggerProductionEvent,
};
use rsim::core::simulation_engine::SimulationEngine;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};

// Custom logger that writes to both console and file
struct FileLogger {
    file: Arc<Mutex<File>>,
}

impl FileLogger {
    fn new(filename: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filename)?;
        Ok(FileLogger {
            file: Arc::new(Mutex::new(file)),
        })
    }

    fn write_line(&self, line: &str) -> std::io::Result<()> {
        if let Ok(mut file) = self.file.lock() {
            writeln!(file, "{}", line)?;
            file.flush()?;
        }
        Ok(())
    }
}

fn run_simulation(
    mode: ProductionMode,
    logger: &FileLogger,
) -> Result<(), Box<dyn std::error::Error>> {
    // Write mode header to log file
    logger.write_line(&format!("\n{}", "=".repeat(80)))?;
    logger.write_line(&format!("SIMULATION MODE: {:?}", mode))?;
    logger.write_line(&format!("{}\n", "=".repeat(80)))?;

    // Create configuration
    let config = BurgerSimulationConfig::new()
        .with_production_mode(mode)
        .with_simulation_duration(200) // Shorter test
        .with_buffer_capacities(5)
        .with_order_quantity_range(1, 2) // Smaller orders
        .with_order_interval(15) // More frequent orders
        .with_random_seed(Some(42))
        .with_metrics_enabled(true)
        .with_event_logging_enabled(true);

    // Validate configuration
    if let Err(e) = config.validate() {
        log::error!("Invalid configuration: {}", e);
        logger.write_line(&format!("ERROR: Invalid configuration: {}", e))?;
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)));
    }

    log::info!("🍔 Burger Production Simulation");
    log::info!("================================");
    log::info!("Mode: {:?}", config.production_mode);
    log::info!("Duration: {} cycles", config.simulation_duration_cycles);
    log::info!(
        "Buffer Capacities: {}",
        config.buffer_capacities.fried_meat_capacity
    );
    log::info!(
        "Order Interval: {} cycles",
        config.order_generation.order_interval_cycles
    );
    log::info!(
        "Order Quantity: {}-{}",
        config.order_generation.min_quantity_per_order,
        config.order_generation.max_quantity_per_order
    );

    // Write config to log file
    logger.write_line("Configuration:")?;
    logger.write_line(&format!("  Mode: {:?}", config.production_mode))?;
    logger.write_line(&format!(
        "  Duration: {} cycles",
        config.simulation_duration_cycles
    ))?;
    logger.write_line(&format!(
        "  Buffer Capacities: {}",
        config.buffer_capacities.fried_meat_capacity
    ))?;
    logger.write_line(&format!(
        "  Order Interval: {} cycles",
        config.order_generation.order_interval_cycles
    ))?;
    logger.write_line(&format!(
        "  Order Quantity: {}-{}",
        config.order_generation.min_quantity_per_order,
        config.order_generation.max_quantity_per_order
    ))?;
    logger.write_line("")?;

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

    // Create metrics collector
    let metrics_collector = MetricsCollector::new("metrics_collector".to_string());

    // Register all components
    engine.register_component(Box::new(fryer)).unwrap();
    engine.register_component(Box::new(baker)).unwrap();
    engine
        .register_component(Box::new(fried_meat_buffer))
        .unwrap();
    engine
        .register_component(Box::new(cooked_bread_buffer))
        .unwrap();
    engine.register_component(Box::new(assembler)).unwrap();
    engine
        .register_component(Box::new(assembly_buffer))
        .unwrap();
    engine.register_component(Box::new(client)).unwrap();
    engine
        .register_component(Box::new(metrics_collector))
        .unwrap();

    // Schedule initial events
    if config.production_mode == ProductionMode::BufferBased {
        // Start production immediately in buffer-based mode
        let fryer_trigger =
            TriggerProductionEvent::new("system".to_string(), Some(vec!["fryer".to_string()]));
        engine.schedule_initial_event(Box::new(fryer_trigger), 1);

        let baker_trigger =
            TriggerProductionEvent::new("system".to_string(), Some(vec!["baker".to_string()]));
        engine.schedule_initial_event(Box::new(baker_trigger), 1);
    }

    // Schedule first order generation
    let first_order = GenerateOrderEvent::new("system".to_string(), None); // Broadcast to all subscribed components
    engine.schedule_initial_event(
        Box::new(first_order),
        config.order_generation.order_interval_cycles,
    );

    // Note: Event logging not available in this SimulationEngine version

    // Run simulation with detailed logging
    log::info!("Starting simulation...");
    let start_time = std::time::Instant::now();

    log::info!("Initial events scheduled:");
    log::info!("- Production triggers at cycle 1");
    log::info!(
        "- First order generation at cycle {}",
        config.order_generation.order_interval_cycles
    );

    // Run simulation using engine.run() - much simpler!
    log::debug!("=== RUNNING SIMULATION ===");
    let final_cycle = engine.run().unwrap();

    let elapsed = start_time.elapsed();

    // Print results
    log::info!("================================");
    log::info!("Simulation Complete!");
    log::info!("================================");
    log::info!("Total cycles simulated: {}", final_cycle);
    log::info!("Real time elapsed: {:.2?}", elapsed);

    // Write results to log file
    logger.write_line("Simulation Results:")?;
    logger.write_line(&format!("  Total cycles simulated: {}", final_cycle))?;
    logger.write_line(&format!("  Real time elapsed: {:.2?}", elapsed))?;
    logger.write_line("")?;

    // Final metrics will be printed automatically when MetricsCollector is dropped

    Ok(())
}

fn main() {
    // Generate a unique timestamp for the log file
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let log_filename = format!("burger_simulation_{}.log", timestamp);

    // Create file logger
    let file_logger = match FileLogger::new(&log_filename) {
        Ok(logger) => logger,
        Err(e) => {
            eprintln!("Failed to create log file: {}", e);
            return;
        }
    };

    let logger_clone = file_logger.file.clone();

    // Initialize logger with custom format that writes to both console and file
    env_logger::Builder::from_default_env()
        .format(move |buf, record| {
            let formatted = format!("[{} {}] {}", record.level(), record.target(), record.args());

            // Write to console
            writeln!(buf, "{}", formatted)?;

            // Also write to file
            if let Ok(mut file) = logger_clone.lock() {
                writeln!(file, "{}", formatted).ok();
                file.flush().ok();
            }

            Ok(())
        })
        .init();

    println!("📝 Logging simulation results to: {}", log_filename);

    // Write header to log file
    file_logger
        .write_line("Burger Production Simulation Log")
        .ok();
    file_logger.write_line(&"=".repeat(80)).ok();

    // Run order-based simulation
    println!("\n🍔 Running ORDER-BASED simulation...");
    if let Err(e) = run_simulation(ProductionMode::OrderBased, &file_logger) {
        eprintln!("Order-based simulation failed: {}", e);
        file_logger
            .write_line(&format!("Order-based simulation failed: {}", e))
            .ok();
    }

    // Add separator
    file_logger
        .write_line(&format!("\n\n{}\n\n", "*".repeat(80)))
        .ok();

    // Run buffer-based simulation
    println!("\n🍔 Running BUFFER-BASED simulation...");
    if let Err(e) = run_simulation(ProductionMode::BufferBased, &file_logger) {
        eprintln!("Buffer-based simulation failed: {}", e);
        file_logger
            .write_line(&format!("Buffer-based simulation failed: {}", e))
            .ok();
    }

    // Final summary
    file_logger
        .write_line(&format!("\n{}", "=".repeat(80)))
        .ok();
    file_logger.write_line("Simulation batch completed").ok();

    println!(
        "\n✅ Both simulations completed. Results saved to: {}",
        log_filename
    );
}
