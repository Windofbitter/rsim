# RSim Core API

RSim is a type-safe, deterministic simulation engine for component-based systems.

## Core API for Developers

### Component Creation

#### Processing Components (Stateless)
```rust
use rsim::core::{
    components::{Component, React, PortType},
    components::module::{ProcessorModule, PortSpec},
};

struct Adder {
    a: i32,
    b: i32,
}

impl React for Adder {
    type Output = i32;
    
    fn react(&mut self) -> Option<Self::Output> {
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
            .map(|(name, t)| PortSpec::input(name)).collect();
        let output_ports = ports.iter().filter(|(_, t)| *t == PortType::Output)
            .map(|(name, t)| PortSpec::output(name)).collect();
        
        ProcessorModule::new(
            "Adder", 
            input_ports, 
            output_ports, 
            vec![], // no memory ports
            |ctx, outputs| {
                let a: i32 = ctx.inputs.get("a")?;
                let b: i32 = ctx.inputs.get("b")?;
                outputs.set("sum", a + b)?;
                Ok(())
            }
        )
    }
}
```

#### Processing Components with Memory Access
```rust
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
            .map(|(name, t)| PortSpec::input(name)).collect();
        let output_ports = ports.iter().filter(|(_, t)| *t == PortType::Output)
            .map(|(name, t)| PortSpec::output(name)).collect();
        let memory_ports = ports.iter().filter(|(_, t)| *t == PortType::Memory)
            .map(|(name, t)| PortSpec::memory(name)).collect();
        
        ProcessorModule::new(
            "MemoryProcessor", 
            input_ports, 
            output_ports, 
            memory_ports,
            |ctx, outputs| {
                // Read from memory (previous cycle data)
                if let Ok(Some(stored_value)) = ctx.memory.read::<i32>("memory", "addr1") {
                    outputs.set("output", stored_value)?;
                }
                
                // Write to memory (affects next cycle)
                if let Ok(input_value) = ctx.inputs.get::<i32>("input") {
                    ctx.memory.write("memory", "addr1", input_value)?;
                }
                
                Ok(())
            }
        )
    }
}
```

#### Memory Components (Stateful)
```rust
use rsim::core::{
    components::{MemoryComponent, Cycle, PortType},
    components::state::MemoryData,
    components::module::MemoryModule,
};

#[derive(Clone)]
struct Buffer {
    data: i32,
}

impl MemoryData for Buffer {}

impl Cycle for Buffer {
    type Output = i32;
    
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
```

### Simulation Setup

```rust
use rsim::core::builder::simulation_builder::Simulation;

// Create simulation
let mut sim = Simulation::new();

// Add components (auto-generates IDs)
let adder1 = sim.add_component(Adder { a: 0, b: 0 });
let adder2 = sim.add_component(Adder { a: 0, b: 0 });
let memory_proc = sim.add_component(MemoryProcessor);
let buffer = sim.add_memory_component(Buffer { data: 0 });

// Connect processor-to-processor (1-to-1 port connections)
sim.connect_component(adder1.output("sum"), adder2.input("a"))?;

// Connect processor to memory (processor memory port -> memory component)
sim.connect_memory(memory_proc.output("memory"), buffer)?;

// Build cycle engine
let cycle_engine = sim.build()?;
```

### Connection Validation

All connections are validated at connection-time with detailed error messages:

```rust
// ✅ Valid connections
sim.connect_component(adder1.output("sum"), adder2.input("a"))?;
sim.connect_memory(memory_proc.output("memory"), buffer)?;

// ❌ These will return validation errors:
sim.connect_component(adder1.output("sum"), adder2.input("a"))?;  // First connection ✓
sim.connect_component(adder1.output("sum"), adder3.input("b"))?;  // ❌ Output already connected

sim.connect_component(adder1.output("result"), adder2.input("x"))?; // First connection ✓
sim.connect_component(adder3.output("value"), adder2.input("x"))?;  // ❌ Input already connected

sim.connect_component(adder1.output("nonexistent"), adder2.input("a"))?; // ❌ Port doesn't exist
```

### Execution

```rust
use rsim::core::execution::cycle_engine::CycleEngine;

let mut engine = cycle_engine;

// Build execution order (topological sort)
engine.build_execution_order()?;

// Run simulation cycles
for _ in 0..10 {
    engine.cycle()?;
}
```

## Core Types

### Events
All component outputs are wrapped in events with timestamps:
```rust
pub struct Event {
    pub timestamp: u64,
    pub event_id: u64,
    pub payload: TypedValue,
}
```

### Component Traits
- **`React`**: Stateless processing logic
- **`Cycle`**: Stateful memory updates  
- **`Component`**: Processing component definition
- **`MemoryComponent`**: Memory component definition (enforces single I/O ports, no memory ports)

### Port Types
```rust
pub enum PortType {
    Input,   // Receives data
    Output,  // Sends data
    Memory,  // Memory access
}
```

## Example: Simple Calculator

```rust
use rsim::core::{
    builder::simulation_builder::Simulation,
    components::{Component, React, PortType},
    components::module::{ProcessorModule, PortSpec},
};

// Define calculator component
struct Calculator;

impl React for Calculator {
    type Output = f64;
    
    fn react(&mut self) -> Option<Self::Output> {
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
            .map(|(name, t)| PortSpec::input(name)).collect();
        let output_ports = ports.iter().filter(|(_, t)| *t == PortType::Output)
            .map(|(name, t)| PortSpec::output(name)).collect();
        
        ProcessorModule::new(
            "Calculator", 
            input_ports, 
            output_ports, 
            vec![], // no memory ports
            |ctx, outputs| {
                let input: f64 = ctx.inputs.get("input")?;
                outputs.set("result", input * 2.0)?;
                Ok(())
            }
        )
    }
}

fn main() -> Result<(), String> {
    // Create simulation
    let mut sim = Simulation::new();
    
    // Add calculator
    let calc = sim.add_component(Calculator);
    
    // No connections needed for this simple example
    
    // Build and run
    let mut engine = sim.build()?;
    engine.build_execution_order()?;
    
    // Execute 5 cycles
    for _ in 0..5 {
        engine.cycle()?;
    }
    
    println!("Simulation completed {} cycles", engine.current_cycle());
    Ok(())
}
```

## Architecture

- **Processing Components**: Stateless, multiple I/O ports, single output type, 1-to-1 port connections
- **Memory Components**: Stateful, exactly one input and one output port, no memory ports, validated at creation
- **Connection Validation**: Real-time validation prevents invalid port connections and duplicates
- **Execution Order**: Topological sorting ensures deterministic execution
- **Memory Model**: Read from previous state, write to current state
- **Type Safety**: Compile-time type checking with runtime type erasure and connection validation

## Key Features

1. **Type Safety**: Automatic type checking for component connections
2. **Connection Validation**: Real-time validation of port connections with detailed error messages
3. **1-to-1 Port Constraints**: Each port connects to exactly one other port (no fan-out/fan-in)
4. **Deterministic**: Topological execution order ensures reproducible results  
5. **Event-Based**: All outputs are timestamped events
6. **Memory Safety**: Double-buffered memory prevents race conditions
7. **Modular**: Components are self-contained and reusable