# RSim Core API

RSim is a type-safe, deterministic simulation engine for component-based systems.

## Core Concepts

### Component Types
- **Processing Components**: Stateless logic with input/output/memory ports
- **Memory Components**: Stateful storage with exactly one input and one output port

### Memory Architecture
Memory components store **structured objects of their own type**, not individual fields. This ensures type safety and proper data encapsulation.

## Quick Start

### Basic Processing Component
```rust
use rsim::*;

#[derive(Debug)]
struct Baker {
    min_delay: u32,
    max_delay: u32,
    seed: u64,
}

impl Baker {
    pub fn new(min_delay: u32, max_delay: u32, seed: u64) -> Self {
        Self { min_delay, max_delay, seed }
    }
}

impl_component!(Baker, "Baker", {
    inputs: [],
    outputs: [],
    memory: [bread_buffer, baker_state],
    react: |ctx, _outputs| {
        use rand::{Rng, SeedableRng};
        use rand::rngs::StdRng;
        
        // Read complete state object from memory
        let mut state = if let Ok(Some(current_state)) = ctx.memory.read::<BakerState>("baker_state", "state") {
            current_state
        } else {
            BakerState::new()
        };
        
        // Read complete buffer object from memory
        let mut buffer = if let Ok(Some(current_buffer)) = ctx.memory.read::<FIFOMemory>("bread_buffer", "buffer") {
            current_buffer
        } else {
            FIFOMemory::new(10)
        };
        
        // Process logic
        if state.remaining_cycles > 0 {
            state.remaining_cycles -= 1;
        } else if !buffer.is_full() {
            buffer.to_add += 1;
            state.total_produced += 1;
            
            let mut rng = StdRng::seed_from_u64(state.rng_state as u64);
            state.remaining_cycles = rng.gen_range(2..=5);
            state.rng_state = rng.next_u64() as i64;
        }
        
        // Write complete objects back to memory
        memory_write!(ctx, "bread_buffer", "buffer", buffer);
        memory_write!(ctx, "baker_state", "state", state);
        
        Ok(())
    }
});
```

### Structured State Components
```rust
#[derive(Clone, Debug)]
pub struct BakerState {
    pub remaining_cycles: i64,
    pub total_produced: i64,
    pub rng_state: i64,
}

impl BakerState {
    pub fn new() -> Self {
        Self {
            remaining_cycles: 0,
            total_produced: 0,
            rng_state: 42,
        }
    }
}

impl rsim::core::components::state::MemoryData for BakerState {}

impl Cycle for BakerState {
    type Output = i64;
    
    fn cycle(&mut self) -> Option<Self::Output> {
        Some(self.total_produced)
    }
}

impl_memory_component!(BakerState, {
    input: input,
    output: output
});
```

### FIFO Memory Component
```rust
#[derive(Clone, Debug)]
pub struct FIFOMemory {
    pub data_count: i64,
    pub to_add: i64,
    pub to_subtract: i64,
    pub capacity: i64,
}

impl FIFOMemory {
    pub fn new(capacity: i64) -> Self {
        Self {
            data_count: 0,
            to_add: 0,
            to_subtract: 0,
            capacity,
        }
    }

    pub fn is_full(&self) -> bool {
        self.data_count >= self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.data_count == 0
    }

    pub fn update(&mut self) {
        self.data_count = self.data_count.saturating_sub(self.to_subtract);
        let can_add = std::cmp::min(self.to_add, self.capacity - self.data_count);
        self.data_count += can_add;
        self.to_add = 0;
        self.to_subtract = 0;
    }
}

impl rsim::core::components::state::MemoryData for FIFOMemory {}

impl Cycle for FIFOMemory {
    type Output = i64;
    
    fn cycle(&mut self) -> Option<Self::Output> {
        self.update();
        Some(self.data_count)
    }
}

impl_memory_component!(FIFOMemory, {
    input: input,
    output: output
});
```

## Simulation Setup

```rust
use rsim::core::builder::simulation_builder::Simulation;

fn main() -> Result<(), String> {
    let mut sim = Simulation::new();
    
    // Add processing components
    let baker = sim.add_component(Baker::new(2, 5, 1000));
    
    // Add memory components
    let bread_buffer = sim.add_memory_component(FIFOMemory::new(10));
    let baker_state = sim.add_memory_component(BakerState::new());
    
    // Connect memory ports
    sim.connect_memory_port(baker.memory_port("bread_buffer"), bread_buffer)?;
    sim.connect_memory_port(baker.memory_port("baker_state"), baker_state)?;
    
    // Build and run
    let mut engine = sim.build()?;
    engine.build_execution_order()?;
    
    for cycle in 1..=100 {
        engine.cycle()?;
        if cycle % 20 == 0 {
            println!("Cycle {}: Running...", cycle);
        }
    }
    
    println!("Completed {} cycles", engine.current_cycle());
    Ok(())
}
```

## Memory Access Patterns

### ✅ Correct: Structured Object Access
```rust
// Read complete object
let mut state = if let Ok(Some(current_state)) = ctx.memory.read::<BakerState>("baker_state", "state") {
    current_state
} else {
    BakerState::new()
};

// Modify object
state.remaining_cycles -= 1;
state.total_produced += 1;

// Write complete object back
memory_write!(ctx, "baker_state", "state", state);
```

### ❌ Incorrect: Individual Field Access
```rust
// This causes type mismatch errors
let cycles = ctx.memory.read::<i64>("baker_state", "remaining_cycles")?;
ctx.memory.write("baker_state", "remaining_cycles", cycles - 1)?;
```

## Available Macros

### Component Definition
- **`impl_component!(Type, "Name", { ... })`** - Implement Component trait
- **`impl_memory_component!(Type, { ... })`** - Implement MemoryComponent trait

### Memory Operations
- **`memory_write!(ctx, "port", "key", value)`** - Write to memory
- **`memory_read!(ctx, "port", "key", var: Type = default)`** - Read with default

## Connection API

```rust
// Processing component to processing component
sim.connect_component(comp1.output("out"), comp2.input("in"))?;

// Processing component to memory component
sim.connect_memory_port(comp.memory_port("mem"), memory_comp)?;
```

## Validation Rules

1. **1-to-1 Connections**: Each port connects to exactly one other port
2. **Type Safety**: Memory components enforce strict type matching
3. **Port Validation**: Connections validated at creation time
4. **Memory Constraints**: Memory components have exactly one input and one output

## Complete Example

See `examples/mc_simulation/` for a comprehensive production line simulation demonstrating:
- Multiple processing components (Baker, Fryer, Assembler, Customer)
- Manager components coordinating data flow
- Structured state memory (BakerState, FryerState, etc.)
- FIFO buffer operations
- Complex component interconnection patterns

## Architecture Principles

- **Deterministic Execution**: Topological ordering ensures reproducible results
- **Type Safety**: Compile-time and runtime type validation
- **Memory Isolation**: Double-buffered memory prevents race conditions
- **Structured Data**: Memory components store cohesive objects, not key-value pairs
- **Connection Constraints**: Enforced 1-to-1 port relationships prevent complex debugging issues