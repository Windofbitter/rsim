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
Type-erased but type-safe container for component values.

```rust
impl TypedValue {
    pub fn new<T: Send + Sync + 'static>(value: T) -> Self;
    pub fn get<T: 'static>(&self) -> Result<&T, String>;
    pub fn into_inner<T: 'static>(self) -> Result<T, String>;
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
        evaluate_fn: fn(&EvaluationContext, &mut TypedOutputMap) -> Result<(), String>,
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
    pub inputs: &'a TypedInputMap,
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
    fn set<T: Send + Sync + 'static>(&mut self, port: &str, value: T) -> Result<(), String>;
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

## Complete Example

```rust
use rsim::core::{
    simulation_builder::SimulationBuilder,
    component_module::{ComponentModule, ProcessorModule, PortSpec, MemoryModule},
    state::MemoryData,
};

// Define custom data type
#[derive(Clone)]
struct ProcessedData {
    value: f64,
    timestamp: u64,
}
impl MemoryData for ProcessedData {}

// Create processor component
fn create_processor() -> ComponentModule {
    ComponentModule::Processing(ProcessorModule::new(
        "processor", 
        vec![PortSpec::input("input")],
        vec![PortSpec::output("processed")],
        vec![PortSpec::memory("cache")],
        |ctx, outputs| {
            // Get input
            let input: f64 = ctx.inputs.get("input")?;
            
            // Process data
            let processed = input * 2.0;
            
            // Store in memory cache
            let cache_data = ProcessedData {
                value: processed,
                timestamp: 123,
            };
            ctx.memory.write("cache", "latest", cache_data)?;
            
            // Set output
            outputs.set("processed", processed)?;
            Ok(())
        }
    ))
}

// Build simulation
let mut simulation = Simulation::new();
simulation.register_module("processor", create_processor())?;
simulation.register_module("memory", ComponentModule::Memory(
    Box::new(MemoryModule::<ProcessedData>::new("cache_memory"))
))?;
let proc_id = simulation.create_component_with_id("processor", "proc_1".to_string())?;
let cache_id = simulation.create_component_with_id("memory", "cache_1".to_string())?;
simulation.connect_memory(proc_id.memory_port("cache"), &cache_id)?;
let cycle_engine = simulation.build()?;

// Run simulation
let mut engine = SimulationEngine::new(cycle_engine, Some(100))?;
let final_cycle = engine.run()?;
```

## Best Practices

1. **Type Safety**: Always use typed inputs/outputs
2. **Error Handling**: Handle all input conditions gracefully  
3. **Memory Usage**: Use memory for persistent state across cycles
4. **Component Isolation**: Keep components focused and minimal
5. **Testing**: Test components individually before integration