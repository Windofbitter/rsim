# Component Connection Refactoring Plan

## Overview
Replace event-based messaging with direct port-to-port connections for core logic and a "probe" system for passive monitoring. This removes the complex Event trait while maintaining decoupled metrics collection.

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
}
```

### Specialized Component Traits
```rust
// For components in the main data path
pub trait ActiveComponent: BaseComponent {
    fn input_ports(&self) -> Vec<&'static str>;
    fn output_port(&self) -> &'static str;
}

pub trait CombinationalComponent: ActiveComponent {
    fn evaluate(&self, port_signals: HashMap<String, Signal>) -> Option<Signal>;
}

pub trait SequentialComponent: ActiveComponent {
    fn current_output(&self) -> Option<Signal>;
    fn prepare_next_state(&mut self, port_signals: &HashMap<String, Signal>);
    fn commit_state_change(&mut self);
}

// For passive, out-of-band monitoring
pub trait ProbeComponent: BaseComponent {
    fn probe(&mut self, signal: &Signal);
}
```

### Component Storage
```rust
pub enum Component {
    Combinational(Box<dyn CombinationalComponent>),
    Sequential(Box<dyn SequentialComponent>),
    Probe(Box<dyn ProbeComponent>),
}
```

## Core Architecture Changes

### Connection Manager
The manager now tracks active connections and passive probes separately.
```rust
pub struct ConnectionManager {
    components: HashMap<ComponentId, Component>,
    // Active data-flow connections: (Source Port) -> Vec<(Target Port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    // Passive monitoring probes: (Source Port) -> Vec<ProbeComponentId>
    probes: HashMap<(ComponentId, String), Vec<ComponentId>>,
    
    combinational_order: Vec<ComponentId>,
    sequential_ids: Vec<ComponentId>,
}

impl ConnectionManager {
    fn connect(&mut self, source: (ComponentId, String), target: (ComponentId, String));
    // Tap into an existing output port for observation
    fn add_probe(&mut self, source_port: (ComponentId, String), probe_id: ComponentId);
    fn build_evaluation_order(&mut self) -> Result<(), String>;
}
```

### Cycle Engine
The engine's main loop is updated to trigger probes after a signal is generated.
```rust
pub struct CycleEngine {
    connection_manager: ConnectionManager,
    current_cycle: u64,
}

impl CycleEngine {
    fn run_cycle(&mut self) {
        // Phase 1: Combinational propagation (topological order)
        // For each component evaluation that produces a signal:
        //   a. Send signal to all connected component inputs.
        //   b. Trigger all probes attached to the output port (pass a clone of the signal).

        // Phase 2: Sequential state preparation
        // Phase 3: Sequential state commit (atomic)
    }
}
```

## Parallel Execution Strategy
Probes do not affect the dependency graph and can be executed concurrently.
```rust
pub struct ParallelCycleEngine {
    dependency_levels: Vec<Vec<ComponentGroup>>,
    thread_pool: ThreadPool,
    // ...
}

impl ParallelCycleEngine {
    fn run_cycle(&mut self) {
        // Phase 1: Parallel combinational evaluation by levels
        for level in &self.dependency_levels {
            self.thread_pool.scope(|scope| {
                for group in level {
                    scope.spawn(|| {
                        // Evaluate component, get output signal
                        // ...
                        // After evaluation, trigger connected probes in parallel
                        for probe in probes_for_output {
                           scope.spawn(|| probe.probe(signal.clone()));
                        }
                    });
                }
            });
        }
        // ... Phases 2 and 3
    }
}
```

## Implementation Steps
1.  Remove `Event` trait and `event_manager.rs`.
2.  Create `Signal` type and new `Component` traits (`ActiveComponent`, `ProbeComponent`).
3.  Implement `ConnectionManager` with `connections` and `probes` maps.
4.  Implement single-threaded `CycleEngine` with three-phase evaluation and probe triggers.
5.  Add topological level analysis to `ConnectionManager`.
6.  Implement `ParallelCycleEngine` with probe support.
7.  Update `SimulationEngine` to use the new cycle engine.
8.  Migrate components to the new `Combinational`, `Sequential`, or `Probe` structure.

## Benefits
- **Simplicity**: No complex event routing for core logic.
- **Performance**: Direct connections for data-flow; no dynamic dispatch.
- **Decoupled Monitoring**: Probes allow metrics/logging without altering core component logic.
- **Hardware Accuracy**: Matches real circuit evaluation with passive probes.
- **Parallelization Ready**: Natural parallel execution model for both components and probes.
- **Scalability**: Performance scales with available threads.
