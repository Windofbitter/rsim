# RSim - Event-Based Discrete Time Simulator

A high-performance, event-driven discrete time simulation framework written in Rust, featuring a complete burger production simulation as a demonstration of manufacturing system modeling and analysis.

## 🎯 Project Overview

RSim provides a clean, extensible framework for building discrete event simulations with deterministic execution, making it ideal for:

- **Manufacturing system analysis** - Production line optimization and bottleneck identification
- **Performance comparison** - Multiple operational modes with comprehensive metrics
- **System design validation** - Event-driven component interaction modeling
- **Educational demonstrations** - Clear separation between simulation framework and domain logic

## 🏗️ Architecture

### Core Simulation Framework (`src/core/`)

- **Event-Driven Architecture**: Components communicate exclusively through typed events
- **Deterministic Execution**: Priority queue with sequence numbers ensures reproducible results
- **Time-Based Scheduling**: Events scheduled with precise cycle delays
- **Component Registry**: Dynamic component registration with subscription-based event routing

### Burger Production Example (`examples/burger_production/`)

A complete manufacturing simulation demonstrating two operational modes:

```
Raw Materials → [Fryer] → Fried Meat Buffer → [Assembler] → Assembly Buffer → [Client]
                    ↓           ↑                    ↑            ↓
Raw Materials → [Baker] → Cooked Bread Buffer -----┘            Orders
```

## 🚀 Key Features

### 📊 **Dual Production Modes**
- **Buffer-Based**: Continuous production with inventory buffers (push system)
- **Order-Based**: On-demand production triggered by customer orders (pull system)

### 📈 **Built-in Metrics Collection**
- Order fulfillment rates and timing analysis
- Production throughput measurement
- Cycle-accurate performance tracking
- Comparative mode analysis

### 🔄 **Advanced Flow Control**
- **Backpressure mechanisms** prevent buffer overflow
- **FIFO queueing** ensures fair resource allocation
- **Event-based coordination** eliminates polling and race conditions

### ⚙️ **Highly Configurable**
- Production delays and buffer capacities
- Order patterns and frequencies
- Simulation duration and random seeds
- Component behavior modes

## 🛠️ Installation & Usage

### Prerequisites
- Rust 1.70+ with Cargo

### Quick Start

```bash
# Clone the repository
git clone <repository-url>
cd rsim

# Run the burger production simulation
cargo run --example burger_production

# Run with detailed logging
RUST_LOG=info cargo run --example burger_production

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt
```

### Example Output

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

## 📖 Configuration

The simulation is highly configurable through `BurgerSimulationConfig`:

```rust
let config = BurgerSimulationConfig::new()
    .with_production_mode(ProductionMode::BufferBased)
    .with_simulation_duration(500)
    .with_buffer_capacities(10)
    .with_order_quantity_range(1, 5)
    .with_order_interval(20)
    .with_random_seed(Some(42));
```

### Key Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `production_mode` | BufferBased or OrderBased | BufferBased |
| `simulation_duration_cycles` | Total simulation time | 200 |
| `buffer_capacities` | FIFO buffer sizes | 5 |
| `order_interval_cycles` | Time between orders | 15 |
| `processing_delays` | Frying/Baking/Assembly times | 10/8/5 |

## 🎯 Performance Insights

The simulation reveals significant performance differences between production modes:

### Buffer-Based Mode (Push System)
- ✅ **Higher throughput**: 2.4x more orders per cycle
- ✅ **Faster fulfillment**: 7.4x lower average response time
- ✅ **Better reliability**: 100% order fulfillment rate
- ❌ **Higher inventory**: Continuous buffer usage

### Order-Based Mode (Pull System)
- ✅ **Lower inventory**: Just-in-time production
- ✅ **Demand-responsive**: No overproduction
- ❌ **Lower throughput**: Limited by production delays
- ❌ **Higher latency**: 19+ cycle average fulfillment time

## 🔧 Framework Components

### Core Types
- **`Event`**: Typed messages with data payloads and routing information
- **`BaseComponent`**: Reactive components with event subscriptions
- **`EventManager`**: Component registry and event routing
- **`EventScheduler`**: Priority queue for time-based event scheduling
- **`SimulationEngine`**: Main orchestration and lifecycle management

### Production Components
- **`Fryer`/`Baker`**: Timed production components with mode-specific behavior
- **`Assembler`**: Reactive assembly component with ingredient coordination
- **`Client`**: Order generation and fulfillment tracking
- **`MetricsCollector`**: Performance analysis and reporting

### Buffer Components
- **FIFO Buffers**: Capacity-limited queues with backpressure signaling
- **Event Broadcasting**: Availability notifications for downstream consumers

## 🎓 Educational Value

This simulation demonstrates key concepts in:

- **Event-Driven Programming**: Decoupled component communication
- **Manufacturing Systems**: Production scheduling and inventory management
- **Performance Analysis**: Quantitative comparison of operational strategies
- **System Design**: Clean separation of concerns and modular architecture

## 🔬 Development

### Project Structure
```
src/core/           # Simulation framework
examples/           # Domain-specific simulations
├── burger_production/
│   ├── components/ # Production and consumer components
│   ├── buffer/     # FIFO buffer implementations
│   ├── events/     # Event type definitions
│   └── config.rs   # Configuration management
tests/              # Unit and integration tests
```

### Extending the Framework

1. **Define Events**: Create typed event structures implementing the `Event` trait
2. **Implement Components**: Build reactive components with `BaseComponent`
3. **Configure Simulation**: Set up component registration and initial events
4. **Run Analysis**: Use built-in metrics or add custom measurement

### Design Principles

- **Deterministic**: Same inputs always produce identical results
- **Modular**: Clear separation between framework and domain logic
- **Observable**: Built-in instrumentation and metrics collection
- **Testable**: Pure functional components with predictable behavior

## 📄 License

This project is open source. See LICENSE file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

---

*Built with ❤️ in Rust - A demonstration of event-driven simulation design and manufacturing system analysis.*