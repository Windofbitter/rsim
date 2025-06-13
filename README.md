# RSim - Event-Based Discrete Time Simulator

A high-performance, event-driven discrete time simulation framework written in Rust. RSim provides a clean, extensible architecture for building complex simulations with deterministic execution and precise timing control.

## 🎯 Framework Overview

RSim is designed for building discrete event simulations with a focus on:

- **Clean Architecture**: Clear separation between simulation framework and domain logic
- **Deterministic Execution**: Reproducible results through priority queues and sequence numbers
- **High Performance**: Efficient event scheduling and batch processing
- **Extensibility**: Modular design allowing custom components and event types
- **Educational Value**: Demonstrates key concepts in event-driven programming and system design

## 🏗️ Core Architecture

### Event-Driven Framework (`src/core/`)

The simulation framework consists of several key components:

- **`SimulationEngine`**: Main orchestration loop that processes events in time order
- **`EventScheduler`**: Priority queue managing time-based event scheduling with deterministic ordering
- **`EventManager`**: Component registry and event routing system
- **`BaseComponent`**: Foundation trait for reactive components with event subscriptions
- **Event System**: Typed events with data payloads and flexible routing

### Core Simulation Flow

1. **Registration**: Components register with EventManager and declare event subscriptions
2. **Scheduling**: EventScheduler maintains a priority queue of events ordered by execution time
3. **Execution Loop**: SimulationEngine orchestrates the main simulation:
   - Pull all events for current time from scheduler
   - Group events by target component
   - Components process events via `react_atomic()` and generate new events
   - New events are scheduled with delays
4. **Continuation**: Simulation continues until no events remain or max cycles reached

### Example Application: Burger Production

A complete manufacturing simulation (`examples/burger_production/`) demonstrates the framework's capabilities:

```
Raw Materials → [Fryer] → Fried Meat Buffer → [Assembler] → Assembly Buffer → [Client]
                    ↓           ↑                    ↑            ↓
Raw Materials → [Baker] → Cooked Bread Buffer -----┘            Orders
```

This example showcases different production modes, buffer management, and performance metrics collection.

## 🚀 Framework Features

### 🎯 **Event-Driven Architecture**
- **Pure Event Communication**: Components interact exclusively through typed events
- **Subscription-Based Routing**: Components declare event interests at registration
- **Deterministic Ordering**: Priority queue with sequence numbers ensures reproducible execution
- **Batch Processing**: All events at the same simulation time are processed together

### ⏰ **Precise Time Management**
- **Discrete Time Simulation**: Cycle-accurate timing with configurable delays
- **Priority Scheduling**: Events processed in strict time order with deterministic tie-breaking
- **Flexible Timing**: Components can schedule events with arbitrary future delays

### 🔧 **Component System**
- **Reactive Components**: Event-driven components with `react_atomic()` processing
- **Modular Design**: Clean separation between framework and application logic
- **Dynamic Registration**: Runtime component registration with flexible configuration
- **State Management**: Built-in component state tracking and lifecycle management

### 📊 **Built-in Observability**
- **Event Tracing**: Comprehensive logging of event processing and timing
- **Component Metrics**: Built-in performance tracking and analysis
- **Deterministic Debugging**: Reproducible execution for reliable testing
- **Configurable Instrumentation**: Adjustable logging levels and metrics collection

## 🛠️ Installation & Usage

### Prerequisites
- Rust 1.70+ with Cargo

### Quick Start

```bash
# Clone the repository
git clone <repository-url>
cd rsim

# Build the framework
cargo build

# Run the burger production example
cargo run --example burger_production

# Run with detailed logging to see event processing
RUST_LOG=info cargo run --example burger_production

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt
```

### Using the Framework

To create your own simulation using RSim:

1. **Define Events**: Create event types implementing the framework's event traits
2. **Implement Components**: Build reactive components using `BaseComponent`
3. **Configure Simulation**: Set up component registration and initial events
4. **Run Simulation**: Use `SimulationEngine` to execute your event-driven simulation

```rust
use rsim::core::{SimulationEngine, EventManager, EventScheduler};

// Set up your simulation components and events
let mut engine = SimulationEngine::new();
// Add your components and initial events
engine.run();
```

## 🔧 Framework Components

### Core Types (`src/core/`)

- **`Event`**: Typed messages with data payloads and routing information
- **`BaseComponent`**: Foundation trait for reactive components with event subscriptions
- **`EventManager`**: Component registry and event routing system
- **`EventScheduler`**: Priority queue for time-based event scheduling
- **`SimulationEngine`**: Main orchestration and lifecycle management
- **`ComponentId`**: String identifier for components
- **`EventType`**: String identifier for event types
- **`ComponentValue`**: Enum for event data (Int, Float, String, Bool)

### Design Principles

- **Deterministic**: Same inputs always produce identical results
- **Modular**: Clear separation between framework and domain logic
- **Observable**: Built-in instrumentation and metrics collection
- **Testable**: Pure functional components with predictable behavior

## 📚 Example: Burger Production Simulation

The `examples/burger_production/` directory contains a complete manufacturing simulation demonstrating the framework's capabilities. This example showcases:

### Production Modes
- **Buffer-Based**: Continuous production with inventory buffers (push system)
- **Order-Based**: On-demand production triggered by customer orders (pull system)

### Key Components
- **Production Components**: `Fryer`, `Baker` with timed processing
- **Assembly Component**: `Assembler` with ingredient coordination
- **Buffer Components**: FIFO queues with backpressure management
- **Client Component**: Order generation and fulfillment tracking
- **Metrics Collection**: Performance analysis and reporting

### Configuration Example

```rust
let config = BurgerSimulationConfig::new()
    .with_production_mode(ProductionMode::BufferBased)
    .with_simulation_duration(500)
    .with_buffer_capacities(10)
    .with_order_quantity_range(1, 5)
    .with_order_interval(20)
    .with_random_seed(Some(42));
```

### Sample Output

```
📊 METRICS SUMMARY
═══════════════════════════════════════
Total Orders Generated: 12
Total Orders Fulfilled: 12
Orders Fulfilled Per Cycle: 0.060
Average Fulfillment Time: 2.58 cycles
Min Fulfillment Time: 1 cycles
Max Fulfillment Time: 10 cycles
Simulation Duration: 200 cycles
═══════════════════════════════════════
```

## 🔬 Development

### Project Structure
```
src/core/           # Event-driven simulation framework
├── component.rs    # BaseComponent trait and state management
├── event.rs        # Event types and traits
├── event_manager.rs# Component registry and event routing
├── event_scheduler.rs# Priority queue for time-based scheduling
├── simulation_engine.rs# Main simulation orchestration
└── types.rs        # Core type definitions

examples/           # Domain-specific simulations
├── burger_production/
│   ├── components/ # Production and consumer components
│   ├── buffer/     # FIFO buffer implementations
│   ├── events/     # Event type definitions
│   └── config.rs   # Configuration management
tests/              # Unit and integration tests
```

### Extending the Framework

1. **Define Events**: Create event types implementing the framework's event traits
2. **Implement Components**: Build reactive components using `BaseComponent`
3. **Register Components**: Set up component registration and subscriptions
4. **Initialize Events**: Schedule initial events to start the simulation
5. **Run Simulation**: Use `SimulationEngine` to execute your event-driven simulation

### Creating Custom Components

```rust
use rsim::core::{BaseComponent, Event, ComponentValue};

struct MyComponent {
    // Component state
}

impl BaseComponent for MyComponent {
    fn react_atomic(&mut self, event: &Event) -> Vec<Event> {
        // Process incoming event and return new events
        vec![]
    }
    
    fn get_subscriptions(&self) -> Vec<String> {
        // Return list of event types this component subscribes to
        vec!["my_event_type".to_string()]
    }
}
```

## 🎓 Applications

RSim is ideal for modeling and analyzing:

- **Manufacturing Systems**: Production lines, supply chains, and process optimization
- **Distributed Systems**: Message passing, coordination protocols, and timing analysis
- **Network Simulations**: Packet routing, congestion control, and performance modeling
- **Queueing Systems**: Service processes, resource allocation, and capacity planning
- **Educational Projects**: Teaching event-driven programming and system design concepts

## 📄 License

This project is open source. See LICENSE file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

---

*Built with ❤️ in Rust - A high-performance event-driven simulation framework for complex system modeling.*