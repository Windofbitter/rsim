use rsim::core::{
    builder::simulation_builder::Simulation,
    components::{Component, React, PortType, MemoryComponent, Cycle},
    components::module::{ProcessorModule, PortSpec},
    components::state::MemoryData,
    values::traits::{EventInputs, EventOutputs},
};

/// Test component: Adder from rsim_core_api.md
struct Adder {
    a: i32,
    b: i32,
}

impl React for Adder {
    type Output = i32;
    
    fn react(&mut self, _ctx: ()) -> Option<Self::Output> {
        Some(self.a + self.b)
    }
}

impl Component for Adder {
    fn define_ports() -> Vec<(String, PortType)> {
        vec![
            ("a".to_string(), PortType::Input),
            ("b".to_string(), PortType::Input),
            ("sum".to_string(), PortType::Output),
        ]
    }
    
    fn into_module() -> ProcessorModule {
        let ports = Self::define_ports();
        let input_ports = ports.iter().filter(|(_, t)| *t == PortType::Input)
            .map(|(name, _)| PortSpec::input(name)).collect();
        let output_ports = ports.iter().filter(|(_, t)| *t == PortType::Output)
            .map(|(name, _)| PortSpec::output(name)).collect();
        
        ProcessorModule::new(
            "Adder", 
            input_ports, 
            output_ports, 
            vec![], // no memory ports
            |ctx, outputs| {
                // Try to get inputs, if not available use default values
                let a: i32 = ctx.inputs.get("a").unwrap_or(0);
                let b: i32 = ctx.inputs.get("b").unwrap_or(0);
                outputs.set("sum", a + b)?;
                Ok(())
            }
        )
    }
}

/// Test component: MemoryProcessor from rsim_core_api.md
struct MemoryProcessor;

impl Component for MemoryProcessor {
    fn define_ports() -> Vec<(String, PortType)> {
        vec![
            ("input".to_string(), PortType::Input),
            ("output".to_string(), PortType::Output),
            ("memory".to_string(), PortType::Memory),  // Memory port
        ]
    }
    
    fn into_module() -> ProcessorModule {
        let ports = Self::define_ports();
        let input_ports = ports.iter().filter(|(_, t)| *t == PortType::Input)
            .map(|(name, _)| PortSpec::input(name)).collect();
        let output_ports = ports.iter().filter(|(_, t)| *t == PortType::Output)
            .map(|(name, _)| PortSpec::output(name)).collect();
        let memory_ports = ports.iter().filter(|(_, t)| *t == PortType::Memory)
            .map(|(name, _)| PortSpec::memory(name)).collect();
        
        ProcessorModule::new(
            "MemoryProcessor", 
            input_ports, 
            output_ports, 
            memory_ports,
            |ctx, outputs| {
                // Read from memory (previous cycle data)
                if let Ok(Some(stored_value)) = ctx.memory.read::<i64>("memory", "addr1") {
                    outputs.set("output", stored_value)?;
                }
                
                // Write to memory (affects next cycle)
                if let Ok(input_value) = ctx.inputs.get::<i64>("input") {
                    ctx.memory.write("memory", "addr1", input_value)?;
                }
                
                Ok(())
            }
        )
    }
}

/// Test component: Buffer memory component from rsim_core_api.md
#[derive(Clone)]
struct Buffer {
    data: i64,
}

impl MemoryData for Buffer {}

impl Cycle for Buffer {
    type Output = i64;
    
    fn cycle(&mut self) -> Option<Self::Output> {
        Some(self.data)
    }
}

impl MemoryComponent for Buffer {
    fn define_ports() -> Vec<(String, PortType)> {
        vec![
            ("input".to_string(), PortType::Input),
            ("output".to_string(), PortType::Output),
        ]
    }
    
    // Note: into_memory_module() is auto-implemented with validation
}

/// Test component: Calculator from rsim_core_api.md
struct Calculator;

impl React for Calculator {
    type Output = f64;
    
    fn react(&mut self, _ctx: ()) -> Option<Self::Output> {
        Some(42.0) // Simple calculation
    }
}

impl Component for Calculator {
    fn define_ports() -> Vec<(String, PortType)> {
        vec![
            ("input".to_string(), PortType::Input),
            ("result".to_string(), PortType::Output),
        ]
    }
    
    fn into_module() -> ProcessorModule {
        let ports = Self::define_ports();
        let input_ports = ports.iter().filter(|(_, t)| *t == PortType::Input)
            .map(|(name, _)| PortSpec::input(name)).collect();
        let output_ports = ports.iter().filter(|(_, t)| *t == PortType::Output)
            .map(|(name, _)| PortSpec::output(name)).collect();
        
        ProcessorModule::new(
            "Calculator", 
            input_ports, 
            output_ports, 
            vec![], // no memory ports
            |ctx, outputs| {
                // Try to get input, if not available use default value
                let input: f64 = ctx.inputs.get("input").unwrap_or(1.0);
                outputs.set("result", input * 2.0)?;
                Ok(())
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adder_component() {
        let mut sim = Simulation::new();
        let adder = sim.add_component(Adder { a: 5, b: 3 });
        
        assert!(sim.has_component(&adder));
        assert_eq!(sim.component_ids().len(), 1);
    }

    #[test]
    fn test_memory_processor_component() {
        let mut sim = Simulation::new();
        let memory_proc = sim.add_component(MemoryProcessor);
        
        assert!(sim.has_component(&memory_proc));
        assert_eq!(sim.component_ids().len(), 1);
    }

    #[test]
    fn test_buffer_memory_component() {
        let mut sim = Simulation::new();
        let buffer = sim.add_memory_component(Buffer { data: 42 });
        
        assert!(sim.has_component(&buffer));
        assert_eq!(sim.component_ids().len(), 1);
    }

    #[test]
    fn test_calculator_component() {
        let mut sim = Simulation::new();
        let calc = sim.add_component(Calculator);
        
        assert!(sim.has_component(&calc));
        assert_eq!(sim.component_ids().len(), 1);
    }

    #[test]
    fn test_component_connections() -> Result<(), String> {
        let mut sim = Simulation::new();
        
        // Add components
        let adder1 = sim.add_component(Adder { a: 0, b: 0 });
        let adder2 = sim.add_component(Adder { a: 0, b: 0 });
        
        // Connect components
        sim.connect_component(adder1.output("sum"), adder2.input("a"))?;
        
        // Verify components were added
        assert_eq!(sim.component_ids().len(), 2);
        assert!(sim.has_component(&adder1));
        assert!(sim.has_component(&adder2));
        
        Ok(())
    }

    #[test]
    fn test_memory_connection() -> Result<(), String> {
        let mut sim = Simulation::new();
        
        // Add components
        let memory_proc = sim.add_component(MemoryProcessor);
        let buffer = sim.add_memory_component(Buffer { data: 0 });
        
        // Connect processor to memory  
        sim.connect_memory(memory_proc.output("memory"), buffer.clone())?;
        
        // Verify components were added
        assert_eq!(sim.component_ids().len(), 2);
        assert!(sim.has_component(&memory_proc));
        assert!(sim.has_component(&buffer));
        
        Ok(())
    }

    #[test]
    fn test_simulation_setup_and_execution() -> Result<(), String> {
        // Create simulation
        let mut sim = Simulation::new();
        
        // Add components (auto-generates IDs)
        let adder1 = sim.add_component(Adder { a: 5, b: 3 });
        let adder2 = sim.add_component(Adder { a: 0, b: 0 });
        let memory_proc = sim.add_component(MemoryProcessor);
        let buffer = sim.add_memory_component(Buffer { data: 100 });
        
        // Connect processor-to-processor (1-to-1 port connections)
        sim.connect_component(adder1.output("sum"), adder2.input("a"))?;
        
        // Connect processor to memory (processor memory port -> memory component)
        sim.connect_memory(memory_proc.output("memory"), buffer.clone())?;
        
        // Build cycle engine
        let mut engine = sim.build()?;
        
        // Build execution order (topological sort)
        engine.build_execution_order()?;
        
        // Run simulation cycles
        for _ in 0..5 {
            engine.cycle()?;
        }
        
        // Verify simulation completed cycles
        assert_eq!(engine.current_cycle(), 5);
        
        Ok(())
    }

    #[test]
    fn test_calculator_full_simulation() -> Result<(), String> {
        // Create simulation
        let mut sim = Simulation::new();
        
        // Add calculator
        let _calc = sim.add_component(Calculator);
        
        // No connections needed for this simple example
        
        // Build and run
        let mut engine = sim.build()?;
        engine.build_execution_order()?;
        
        // Execute 5 cycles
        for _ in 0..5 {
            engine.cycle()?;
        }
        
        // Verify simulation completed the expected number of cycles
        assert_eq!(engine.current_cycle(), 5);
        
        Ok(())
    }

    #[test]
    fn test_connection_validation_errors() {
        let mut sim = Simulation::new();
        let adder1 = sim.add_component(Adder { a: 0, b: 0 });
        let adder2 = sim.add_component(Adder { a: 0, b: 0 });
        let adder3 = sim.add_component(Adder { a: 0, b: 0 });
        
        // First connection should succeed
        assert!(sim.connect_component(adder1.output("sum"), adder2.input("a")).is_ok());
        
        // Second connection to same output should fail
        assert!(sim.connect_component(adder1.output("sum"), adder3.input("b")).is_err());
        
        // Connection to same input should fail
        assert!(sim.connect_component(adder3.output("sum"), adder2.input("a")).is_err());
        
        // Connection to nonexistent port should fail
        assert!(sim.connect_component(adder1.output("nonexistent"), adder2.input("a")).is_err());
    }
}