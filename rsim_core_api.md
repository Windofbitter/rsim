# RSim Core API Documentation

RSim is a type-safe, deterministic simulation engine built in Rust.

## Quick Start

```rust
use rsim::core::{
    Simulation,
    simulation_engine::SimulationEngine,
    component_module::{ComponentModule, ProcessorModule, PortSpec},
};

// Create a simple simulation
let mut simulation = Simulation::new();
simulation.register_module("adder", create_adder_module())?;
let adder1 = simulation.create_component("adder")?;
let adder2 = simulation.create_component("adder")?;
simulation.connect(adder1.output("result"), adder2.input("input_a"))?;
let cycle_engine = simulation.build()?;

let mut engine = SimulationEngine::new(cycle_engine, Some(10))?;
let final_cycle = engine.run()?;
```

## Core Types

### TypedValue
Type-erased but type-safe container for component values. Supports any `Clone` type automatically.

```rust
impl TypedValue {
    pub fn new<T: Send + Sync + Clone + 'static>(value: T) -> Self;
    pub fn get<T: 'static>(&self) -> Result<&T, String>;
    pub fn into_inner<T: 'static>(self) -> Result<T, String>;
}
```

### Event
Event wrapper with timestamp, unique ID, and typed payload for timing-aware simulations.

```rust
impl Event {
    pub fn new<T: Send + Sync + Clone + 'static>(timestamp: u64, payload: T) -> Self;
    pub fn get_payload<T: 'static>(&self) -> Result<&T, String>;
    pub fn into_payload<T: 'static>(self) -> Result<T, String>;
    
    pub event_id: u64;        // Unique identifier
    pub timestamp: u64;       // Simulation cycle timestamp
}
```

## Simulation API

Imperative API for constructing simulations with component ID handles.

```rust
impl Simulation {
    pub fn new() -> Self;
    
    // Module registration
    pub fn register_module(&mut self, name: &str, module: ComponentModule) -> Result<(), String>;
    
    // Component creation - returns ComponentId handles
    pub fn create_component(&mut self, module_name: &str) -> Result<ComponentId, String>;
    pub fn create_component_with_id(&mut self, module_name: &str, id: String) -> Result<ComponentId, String>;
    pub fn create_components(&mut self, module_name: &str, count: usize) -> Result<Vec<ComponentId>, String>;
    
    // Type-safe connections using port handles
    pub fn connect(&mut self, output: OutputPort, input: InputPort) -> Result<(), String>;
    pub fn connect_memory(&mut self, memory_port: MemoryPort, memory_component: &ComponentId) -> Result<(), String>;
    
    // Build
    pub fn build(self) -> Result<CycleEngine, String>;
}
```

## Component System

### ProcessorModule

```rust
impl ProcessorModule {
    pub fn new(
        name: &str,
        input_ports: Vec<PortSpec>,
        output_ports: Vec<PortSpec>,
        memory_ports: Vec<PortSpec>,
        evaluate_fn: fn(&EvaluationContext, &mut EventOutputMap) -> Result<(), String>,
    ) -> Self;
}
```

### PortSpec

```rust
impl PortSpec {
    pub fn input(name: &str) -> Self;           // Required input port
    pub fn input_optional(name: &str) -> Self;  // Optional input port
    pub fn output(name: &str) -> Self;          // Output port
    pub fn memory(name: &str) -> Self;          // Memory port
}
```

### EvaluationContext

```rust
pub struct EvaluationContext<'a> {
    pub inputs: &'a EventInputMap,
    pub memory: &'a mut TypeSafeCentralMemoryProxy,
    pub state: Option<&'a mut dyn ComponentState>,
    pub component_id: &'a ComponentId,
}
```

## Type Safety

### TypedInputs & TypedOutputs

```rust
pub trait TypedInputs {
    fn get<T: 'static + Clone>(&self, port: &str) -> Result<T, String>;
    fn has_input(&self, port: &str) -> bool;
}

pub trait TypedOutputs {
    fn set<T: Send + Sync + Clone + 'static>(&mut self, port: &str, value: T) -> Result<(), String>;
}
```

### EventInputs & EventOutputs (Progressive Disclosure API)

```rust
pub trait EventInputs {
    // Simple: Just get the value (90% of use cases)
    fn get<T: 'static + Clone>(&self, port: &str) -> Result<T, String>;
    
    // Intermediate: Access timing information
    fn get_timestamp(&self, port: &str) -> Result<u64, String>;
    
    // Advanced: Full event access
    fn get_event(&self, port: &str) -> Result<&Event, String>;
    
    fn has_input(&self, port: &str) -> bool;
}

pub trait EventOutputs {
    // Simple: Set value (Event creation automatic)
    fn set<T: Send + Sync + Clone + 'static>(&mut self, port: &str, value: T) -> Result<(), String>;
    
    // Advanced: Emit event directly
    fn emit_event(&mut self, port: &str, event: Event) -> Result<(), String>;
}
```

## Memory System

### TypeSafeMemoryProxy

```rust
pub trait TypeSafeMemoryProxy {
    fn read<T: MemoryData>(&self, port: &str, address: &str) -> Result<Option<T>, String>;
    fn write<T: MemoryData>(&mut self, port: &str, address: &str, data: T) -> Result<(), String>;
}
```

## Execution Engine

### SimulationEngine

```rust
impl SimulationEngine {
    pub fn new(cycle_engine: CycleEngine, max_cycles: Option<u64>) -> Result<Self, String>;
    pub fn run(&mut self) -> Result<u64, String>;
    pub fn step(&mut self) -> Result<(), String>;
    pub fn current_cycle(&self) -> u64;
}
```

## Event-Based Component Examples

### Simple Event Component (90% of use cases)
```rust
fn simple_adder(ctx: &EvaluationContext, outputs: &mut EventOutputMap) -> Result<(), String> {
    // Just use .get() - no need to know about Events
    let a = ctx.inputs.get::<i64>("input_a")?;
    let b = ctx.inputs.get::<i64>("input_b")?;
    
    // Event creation is automatic
    outputs.set("output", a + b)?;
    Ok(())
}
```

### Timestamp-Aware Component
```rust
fn delay_component(ctx: &EvaluationContext, outputs: &mut EventOutputMap) -> Result<(), String> {
    let data = ctx.inputs.get::<i64>("data")?;
    let input_timestamp = ctx.inputs.get_timestamp("data")?;
    
    // Only process data older than 5 cycles
    if input_timestamp <= 5 {
        return Ok(()); // Skip recent data
    }
    
    outputs.set("delayed_output", data)?;
    Ok(())
}
```

### Event Correlation Component
```rust
fn event_correlator(ctx: &EvaluationContext, outputs: &mut EventOutputMap) -> Result<(), String> {
    if ctx.inputs.has_input("sensor_a") && ctx.inputs.has_input("sensor_b") {
        let timestamp_a = ctx.inputs.get_timestamp("sensor_a")?;
        let timestamp_b = ctx.inputs.get_timestamp("sensor_b")?;
        
        // Only correlate synchronized events
        if timestamp_a == timestamp_b {
            let data_a = ctx.inputs.get::<f64>("sensor_a")?;
            let data_b = ctx.inputs.get::<f64>("sensor_b")?;
            outputs.set("correlation", data_a * data_b)?;
        }
    }
    Ok(())
}
```

## Custom Types Example

```rust
use rsim::core::{
    Simulation,
    simulation_engine::SimulationEngine,
    component_module::{ComponentModule, ProcessorModule, PortSpec},
    typed_values::{EventInputs, EventOutputs},
};

// Define custom data types - just implement Clone!
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

// Create components using custom types
fn create_data_source() -> ComponentModule {
    ComponentModule::Processing(ProcessorModule::new(
        "data_source",
        vec![],
        vec![PortSpec::output("data")],
        vec![],
        |_ctx, outputs| {
            let reading = SensorReading {
                value: 42.0,
                sensor_id: 1,
                timestamp: 123,
            };
            outputs.set("data", reading)?; // Custom types just work! ✨
            Ok(())
        }
    ))
}

fn create_processor() -> ComponentModule {
    ComponentModule::Processing(ProcessorModule::new(
        "processor",
        vec![PortSpec::input("sensor_data")],
        vec![PortSpec::output("processed")],
        vec![],
        |ctx, outputs| {
            let reading = ctx.inputs.get::<SensorReading>("sensor_data")?;
            
            let processed = ProcessedReading {
                avg: reading.value * 2.0,
                count: 1,
                timestamp: reading.timestamp,
            };
            
            outputs.set("processed", processed)?;
            Ok(())
        }
    ))
}

// Build and run simulation
let mut simulation = Simulation::new();
simulation.register_module("data_source", create_data_source())?;
simulation.register_module("processor", create_processor())?;

let source = simulation.create_component("data_source")?;
let processor = simulation.create_component("processor")?;
simulation.connect(source.output("data"), processor.input("sensor_data"))?;

let cycle_engine = simulation.build()?;
let mut engine = SimulationEngine::new(cycle_engine, Some(10))?;
let final_cycle = engine.run()?;
```

## Best Practices

1. **Custom Types**: Just implement `Clone` - types work automatically with RSim
2. **Type Safety**: Always use typed inputs/outputs for compile-time guarantees
3. **Error Handling**: Handle all input conditions gracefully  
4. **Component Isolation**: Keep components focused and minimal
5. **Event Usage**: Use simple `.get()` for most cases, timestamps for timing logic, full events for advanced scenarios
6. **Testing**: Test components individually before integration