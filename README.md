# RSim

A type-safe, deterministic simulation engine for component-based systems in Rust.

## Overview

RSim enables building complex simulations through composable components with guaranteed deterministic execution. Components communicate through typed ports with strict 1-to-1 connection constraints, ensuring predictable and debuggable simulations.

## Key Features

- **Type Safety**: Compile-time and runtime type validation for all component connections
- **Deterministic**: Topological execution ordering ensures reproducible results across runs
- **Memory Safety**: Double-buffered memory system prevents race conditions
- **Component-Based**: Modular architecture with reusable processing and memory components
- **Connection Validation**: Real-time validation prevents invalid port connections
- **Macro System**: Dramatically reduces boilerplate code for component definitions

## Quick Start

### Installation

Add RSim to your `Cargo.toml`:

```toml
[dependencies]
rsim = { path = "." }  # For local development
```

### Basic Example

```rust
use rsim::*;

// Define a processing component
#[derive(Debug)]
struct Calculator;

impl_component!(Calculator, "Calculator", {
    inputs: [a, b],
    outputs: [result],
    memory: [],
    react: |ctx, outputs| {
        let a: f64 = ctx.inputs.get("a").unwrap_or_default();
        let b: f64 = ctx.inputs.get("b").unwrap_or_default();
        outputs.set("result", a + b)?;
        Ok(())
    }
});

fn main() -> Result<(), String> {
    let mut sim = Simulation::new();
    let calc = sim.add_component(Calculator);
    
    let mut engine = sim.build()?;
    engine.build_execution_order()?;
    
    for _ in 0..10 {
        engine.cycle()?;
    }
    
    println!("Simulation completed {} cycles", engine.current_cycle());
    Ok(())
}
```

## Architecture

### Component Types

- **Processing Components**: Stateless logic with input/output/memory ports
- **Memory Components**: Stateful storage with exactly one input and one output port

### Memory Model

Memory components store **structured objects of their own type**, ensuring type safety:

```rust
// ✅ Correct: Structured state access
let mut state = ctx.memory.read::<BakerState>("state", "data")?;
state.counter += 1;
memory_write!(ctx, "state", "data", state);

// ❌ Incorrect: Individual field access
ctx.memory.write("state", "counter", counter + 1)?; // Type mismatch error
```

## Complete Example: McDonald's Simulation

The `examples/mc_simulation/` directory contains a comprehensive production line simulation featuring:

- **10 Bakers** producing bread with randomized timing
- **10 Fryers** producing meat patties 
- **Manager Components** coordinating ingredient distribution
- **10 Assemblers** creating burgers from ingredients
- **10 Customers** consuming the final products
- **FIFO Buffers** managing production flow between stages

```bash
cargo run --bin mcdonald_main
```

This demonstrates:
- Complex component interconnection patterns
- Structured state memory (`BakerState`, `FryerState`, etc.)
- FIFO buffer operations with proper type handling
- Multi-stage production pipeline coordination

## Documentation

- **[Core API Reference](rsim_core_api.md)** - Complete technical documentation
- **[Examples](examples/)** - Working simulation examples
- **[McDonald's Design](MCDONALD_SIMULATION_DESIGN.md)** - Production line architecture

## Development

### Running Tests

```bash
cargo test
```

### Running Examples

```bash
# McDonald's production simulation
cargo run --bin mcdonald_main

# Basic component tests
cargo run --example mc_simulation
```

### Building

```bash
cargo build --release
```

## Project Structure

```
rsim/
├── src/
│   ├── core/           # Core simulation engine
│   ├── components/     # Component trait definitions
│   └── macros/         # Code generation macros
├── examples/
│   └── mc_simulation/  # McDonald's production line example
├── rsim_core_api.md    # Technical documentation
└── README.md           # This file
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

Built with Rust's type system to ensure simulation correctness and performance.