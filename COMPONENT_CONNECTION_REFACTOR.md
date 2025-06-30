# Component Connection Refactoring Plan

## Overview
Replace event-based messaging with direct port-to-port connections and typed signals. Remove complex Event trait from core framework.

## Signal Model
```rust
// Core provides minimal signal abstraction
pub type Signal = Box<dyn std::any::Any + Send>;

// Users define concrete signal types
#[derive(Debug, Clone)]
enum UserSignal {
    Trigger,
    DataReady { value: i32 },
    Error { message: String },
}
```

## Component Model

### Base Component Trait
```rust
pub trait BaseComponent: Send {
    fn component_id(&self) -> &ComponentId;
    fn input_ports(&self) -> Vec<&'static str>;
    fn output_port(&self) -> &'static str;
}
```

### Specialized Component Traits
```rust
pub trait CombinationalComponent: BaseComponent {
    fn evaluate(&self, port_signals: HashMap<String, Signal>) -> Option<Signal>;
}

pub trait SequentialComponent: BaseComponent {
    fn current_output(&self) -> Option<Signal>;
    fn prepare_next_state(&mut self, port_signals: &HashMap<String, Signal>);
    fn commit_state_change(&mut self);
}
```

### Component Storage
```rust
pub enum Component {
    Combinational(Box<dyn CombinationalComponent>),
    Sequential(Box<dyn SequentialComponent>),
}
```

## Core Architecture Changes

### Replace Files
- `event.rs` → Remove entirely
- `event_manager.rs` → `connection_manager.rs`
- `event_scheduler.rs` → `cycle_engine.rs`
- `simulation_engine.rs` → Simplified to use CycleEngine

### Connection Manager
```rust
pub struct ConnectionManager {
    components: HashMap<ComponentId, Component>,
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    combinational_order: Vec<ComponentId>,
    sequential_ids: Vec<ComponentId>,
}

impl ConnectionManager {
    fn connect(&mut self, source: (ComponentId, String), target: (ComponentId, String));
    fn build_evaluation_order(&mut self) -> Result<(), String>;
}
```

### Cycle Engine
```rust
pub struct CycleEngine {
    connection_manager: ConnectionManager,
    current_cycle: u64,
}

impl CycleEngine {
    fn run_cycle(&mut self) {
        // Phase 1: Combinational propagation (topological order)
        // Phase 2: Sequential state preparation
        // Phase 3: Sequential state commit (atomic)
    }
}
```

## Implementation Steps
1. Remove Event trait and dynamic routing system
2. Create Signal type and ConnectionManager
3. Implement CycleEngine with three-phase evaluation
4. Update SimulationEngine to use cycles instead of events
5. Migrate components to new trait structure

## Benefits
- **Simplicity**: No complex event routing or subscriptions
- **Performance**: Direct connections, no dynamic dispatch
- **Type Safety**: Users control signal types at compile time
- **Hardware Accuracy**: Matches real circuit evaluation
- **User Control**: Framework provides structure, users define semantics