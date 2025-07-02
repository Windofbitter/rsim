/// Example demonstrating the new imperative Simulation API
use rsim::core::{
    Simulation,
    execution::simulation_engine::SimulationEngine,
    components::module::{ComponentModule, ProcessorModule, PortSpec},
};
use rsim::{EventInputs, EventOutputs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create adder module
    fn create_adder_module() -> ComponentModule {
        ComponentModule::Processing(ProcessorModule::new(
            "adder",
            vec![
                PortSpec::input("input_a"),
                PortSpec::input("input_b"),
            ],
            vec![PortSpec::output("result")],
            vec![],
            |ctx, outputs| {
                let a: f64 = ctx.inputs.get("input_a")?;
                let b: f64 = ctx.inputs.get("input_b")?;
                let result = a + b;
                outputs.set("result", result)?;
                Ok(())
            },
        ))
    }

    // Create source module
    fn create_source_module() -> ComponentModule {
        ComponentModule::Processing(ProcessorModule::new(
            "source",
            vec![],
            vec![PortSpec::output("value")],
            vec![],
            |_ctx, outputs| {
                outputs.set("value", 5.0_f64)?;
                Ok(())
            },
        ))
    }

    // Create sink module
    fn create_sink_module() -> ComponentModule {
        ComponentModule::Processing(ProcessorModule::new(
            "sink",
            vec![PortSpec::input("input")],
            vec![],
            vec![],
            |ctx, _outputs| {
                let value: f64 = ctx.inputs.get("input")?;
                println!("Sink received: {}", value);
                Ok(())
            },
        ))
    }

    // Create simulation using new imperative API
    let mut simulation = Simulation::new();
    
    // Register modules
    simulation.register_module("adder", create_adder_module())?;
    simulation.register_module("source", create_source_module())?;
    simulation.register_module("sink", create_sink_module())?;
    
    // Create components and store their IDs
    let source1 = simulation.create_component("source")?;
    let source2 = simulation.create_component("source")?;
    let adder = simulation.create_component("adder")?;
    let sink = simulation.create_component("sink")?;
    
    // Connect components using port handles
    simulation.connect(source1.output("value"), adder.input("input_a"))?;
    simulation.connect(source2.output("value"), adder.input("input_b"))?;
    simulation.connect(adder.output("result"), sink.input("input"))?;
    
    // Build the simulation
    let cycle_engine = simulation.build()?;
    
    // Run simulation
    let mut engine = SimulationEngine::new(cycle_engine, Some(5))?;
    let final_cycle = engine.run()?;
    
    println!("Simulation completed after {} cycles", final_cycle);
    
    Ok(())
}