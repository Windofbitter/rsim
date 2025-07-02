# Component Pipeline Test

A comprehensive test design for RSim's component pipeline functionality demonstrating data flow, type safety, and memory operations.

## Architecture

```
[Data Source] → [Filter] → [Aggregator] → [Memory]
                    ↓          ↓            ↓
               (validates)  (computes)   (stores)
               SensorReading  running avg  ProcessedReading
```

## Data Types

```rust
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

impl MemoryData for SensorReading {}
impl MemoryData for ProcessedReading {}
```

## Component Definitions

### 1. Data Source
- **Inputs**: None
- **Outputs**: `data` (SensorReading)
- **Function**: Generates sensor readings each cycle

### 2. Filter
- **Inputs**: `raw_data` (SensorReading)
- **Outputs**: `filtered` (SensorReading)
- **Function**: Validates readings, removes outliers (value > 100.0)

### 3. Aggregator
- **Inputs**: `data_stream` (SensorReading)
- **Outputs**: None
- **Memory**: `results` (ProcessedReading)
- **Function**: Computes running averages, stores results

### 4. Memory
- **Type**: Standard memory module
- **Data**: ProcessedReading storage

## Test Setup

```rust
let mut simulation = Simulation::new();

// Register modules
simulation.register_module("data_source", create_data_source_module());
simulation.register_module("filter", create_filter_module());
simulation.register_module("aggregator", create_aggregator_module());
simulation.register_module("memory", create_memory_module());

// Create components
let source = simulation.create_component("data_source")?;
let filter = simulation.create_component("filter")?;
let aggregator = simulation.create_component("aggregator")?;
let memory = simulation.create_component("memory")?;

// Connect pipeline
simulation.connect(source.output("data"), filter.input("raw_data"))?;
simulation.connect(filter.output("filtered"), aggregator.input("data_stream"))?;
simulation.connect_memory(aggregator.memory_port("results"), &memory)?;

// Build and run
let cycle_engine = simulation.build()?;
let mut engine = SimulationEngine::new(cycle_engine, Some(20))?;
let final_cycle = engine.run()?;
```

## Test Scenarios

### A. Basic Flow Test
- Data source generates readings with values 10.0, 25.0, 50.0, 75.0
- Filter passes valid readings (< 100.0)
- Aggregator computes running average: 10.0, 17.5, 28.3, 40.0
- Memory stores ProcessedReading results

### B. Error Condition Tests
- Data source generates invalid reading (value > 100.0)
- Filter blocks invalid reading
- Aggregator handles missing input gracefully
- Verify error propagation through pipeline

### C. Memory Persistence Test
- Run multiple cycles
- Verify aggregator maintains running state
- Check memory contains historical ProcessedReading entries

## Validation Points

✅ **Type Safety**: SensorReading → Filter → Aggregator  
✅ **Component Isolation**: Each component operates independently  
✅ **Memory Operations**: Write/read ProcessedReading data  
✅ **Pipeline Flow**: Data flows correctly through stages  
✅ **Error Handling**: Invalid data rejected by filter  
✅ **Multi-cycle Execution**: Persistent state across cycles

## Expected Results

After 10 cycles with valid inputs:
- Memory contains 10 ProcessedReading entries
- Final running average reflects all processed values
- No errors in pipeline execution
- Deterministic behavior on repeated runs