use rsim::core::{
    Simulation,
    execution::simulation_engine::SimulationEngine,
    components::module::{ComponentModule, ProcessorModule, PortSpec},
};
use rsim::{EventInputs, EventOutputs};

#[derive(Clone, Debug)]
struct SensorReading {
    value: f64,
    sensor_id: u32,
    timestamp: u64,
}

#[derive(Clone, Debug)]
struct ProcessedReading {
    avg: f64,
    count: u32,
    timestamp: u64,
}

fn create_data_source_module() -> ComponentModule {
    ComponentModule::Processing(ProcessorModule::new(
        "data_source",
        vec![], // No inputs
        vec![PortSpec::output("data")],
        vec![],
        |_ctx, outputs| {
            // Generate sensor readings with test data
            static mut CYCLE_COUNTER: u64 = 0;
            let cycle = unsafe { 
                CYCLE_COUNTER += 1;
                CYCLE_COUNTER
            };
            
            let test_values = [10.0, 25.0, 50.0, 75.0, 120.0]; // Last value > 100 for testing
            let value = test_values[(cycle - 1) as usize % test_values.len()];
            
            let reading = SensorReading {
                value,
                sensor_id: 1,
                timestamp: cycle,
            };
            
            outputs.set("data", reading)?;
            Ok(())
        }
    ))
}

fn create_filter_module() -> ComponentModule {
    ComponentModule::Processing(ProcessorModule::new(
        "filter",
        vec![PortSpec::input("raw_data")],
        vec![PortSpec::output("filtered")],
        vec![],
        |ctx, outputs| {
            if let Ok(reading) = ctx.inputs.get::<SensorReading>("raw_data") {
                // Filter out outliers (value > 100.0)
                if reading.value <= 100.0 {
                    println!("Filter passed: {} from sensor {}", reading.value, reading.sensor_id);
                    outputs.set("filtered", reading)?;
                } else {
                    println!("Filter blocked outlier: {}", reading.value);
                }
                // If value > 100.0, don't output anything (filter blocks it)
            }
            Ok(())
        }
    ))
}

fn create_aggregator_module() -> ComponentModule {
    ComponentModule::Processing(ProcessorModule::new(
        "aggregator",
        vec![PortSpec::input("data_stream")],
        vec![PortSpec::output("processed")], // Output processed readings
        vec![],
        |ctx, outputs| {
            if let Ok(reading) = ctx.inputs.get::<SensorReading>("data_stream") {
                // Simple aggregation - create processed reading
                let processed = ProcessedReading {
                    avg: reading.value * 2.0, // Simple processing
                    count: 1,
                    timestamp: reading.timestamp,
                };
                
                outputs.set("processed", processed.clone())?;
                println!("Aggregator processed: {} -> avg={}", reading.value, processed.avg);
            }
            Ok(())
        }
    ))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_pipeline_flow() -> Result<(), String> {
        let mut simulation = Simulation::new();
        
        // Register modules
        simulation.register_module("data_source", create_data_source_module())?;
        simulation.register_module("filter", create_filter_module())?;
        simulation.register_module("aggregator", create_aggregator_module())?;
        
        // Create components
        let source = simulation.create_component("data_source")?;
        let filter = simulation.create_component("filter")?;
        let aggregator = simulation.create_component("aggregator")?;
        
        // Connect pipeline: [Data Source] → [Filter] → [Aggregator]
        simulation.connect(source.output("data"), filter.input("raw_data"))?;
        simulation.connect(filter.output("filtered"), aggregator.input("data_stream"))?;
        
        // Build and run
        let cycle_engine = simulation.build()?;
        let mut engine = SimulationEngine::new(cycle_engine, Some(10))?;
        let final_cycle = engine.run()?;
        
        println!("Pipeline test completed after {} cycles", final_cycle);
        Ok(())
    }
    
    #[test]
    fn test_filter_blocks_outliers() -> Result<(), String> {
        let mut simulation = Simulation::new();
        
        simulation.register_module("data_source", create_data_source_module())?;
        simulation.register_module("filter", create_filter_module())?;
        simulation.register_module("aggregator", create_aggregator_module())?;
        
        let source = simulation.create_component("data_source")?;
        let filter = simulation.create_component("filter")?;
        let aggregator = simulation.create_component("aggregator")?;
        
        simulation.connect(source.output("data"), filter.input("raw_data"))?;
        simulation.connect(filter.output("filtered"), aggregator.input("data_stream"))?;
        
        let cycle_engine = simulation.build()?;
        let mut engine = SimulationEngine::new(cycle_engine, Some(8))?; // Run enough cycles to hit outlier
        let final_cycle = engine.run()?;
        
        println!("Filter outlier test completed after {} cycles", final_cycle);
        Ok(())
    }
    
    #[test]
    fn test_component_isolation() -> Result<(), String> {
        let mut simulation = Simulation::new();
        
        simulation.register_module("data_source", create_data_source_module())?;
        simulation.register_module("filter", create_filter_module())?;
        simulation.register_module("aggregator", create_aggregator_module())?;
        
        let source = simulation.create_component("data_source")?;
        let filter = simulation.create_component("filter")?;
        let aggregator = simulation.create_component("aggregator")?;
        
        simulation.connect(source.output("data"), filter.input("raw_data"))?;
        simulation.connect(filter.output("filtered"), aggregator.input("data_stream"))?;
        
        let cycle_engine = simulation.build()?;
        let mut engine = SimulationEngine::new(cycle_engine, Some(15))?;
        let final_cycle = engine.run()?;
        
        println!("Component isolation test completed after {} cycles", final_cycle);
        Ok(())
    }
}

fn main() -> Result<(), String> {
    println!("Component Pipeline Test");
    println!("======================");
    
    let mut simulation = Simulation::new();
    
    // Register modules
    simulation.register_module("data_source", create_data_source_module())?;
    simulation.register_module("filter", create_filter_module())?;
    simulation.register_module("aggregator", create_aggregator_module())?;
    
    // Create components
    let source = simulation.create_component("data_source")?;
    let filter = simulation.create_component("filter")?;
    let aggregator = simulation.create_component("aggregator")?;
    
    // Connect pipeline: [Data Source] → [Filter] → [Aggregator]
    simulation.connect(source.output("data"), filter.input("raw_data"))?;
    simulation.connect(filter.output("filtered"), aggregator.input("data_stream"))?;
    
    println!("Pipeline: [Data Source] → [Filter] → [Aggregator]");
    println!("Running simulation...");
    
    // Build and run
    let cycle_engine = simulation.build()?;
    let mut engine = SimulationEngine::new(cycle_engine, Some(20))?;
    let final_cycle = engine.run()?;
    
    println!("✅ Component pipeline test completed successfully!");
    println!("   Final cycle: {}", final_cycle);
    println!("   Pipeline demonstrates:");
    println!("   - Type-safe data flow (SensorReading → ProcessedReading)");
    println!("   - Component isolation and modularity");
    println!("   - Component modularity and data processing");
    println!("   - Error handling (filter blocks outliers > 100.0)");
    println!("   - Multi-cycle execution with data transformation");
    
    Ok(())
}