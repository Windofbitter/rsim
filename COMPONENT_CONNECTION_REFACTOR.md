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

## Parallel Execution Strategy

### Topological Level Partitioning
```rust
pub struct ParallelCycleEngine {
    dependency_levels: Vec<Vec<ComponentGroup>>,  // Topological levels
    thread_pool: ThreadPool,
    current_cycle: u64,
}

impl ParallelCycleEngine {
    fn run_cycle(&mut self) {
        // Phase 1: Parallel combinational evaluation by levels
        for level in &self.dependency_levels {
            self.thread_pool.scope(|scope| {
                for group in level {
                    scope.spawn(|| group.evaluate_combinational());
                }
            }); // Implicit barrier - all threads join here
        }
        
        // Phase 2: Sequential state preparation (parallel within level)
        for level in &self.sequential_levels {
            self.thread_pool.scope(|scope| {
                for group in level {
                    scope.spawn(|| group.prepare_sequential_state());
                }
            });
        }
        
        // Phase 3: Sequential state commit (atomic, single-threaded)
        self.commit_all_sequential_states();
    }
}
```

### Component Grouping Strategy
```rust
pub struct ComponentGroup {
    components: Vec<ComponentId>,
    internal_connections: HashMap<ComponentId, Vec<ComponentId>>,
    external_inputs: Vec<(ComponentId, String)>,
    external_outputs: Vec<(ComponentId, String)>,
}

impl ComponentGroup {
    fn evaluate_combinational(&mut self) {
        // Evaluate components in topological order within group
        // All dependencies within group are resolved internally
        for comp_id in &self.topological_order {
            // Safe to evaluate - all inputs either external (already available) 
            // or from earlier components in this group
        }
    }
}
```

### Parallelization Benefits
- **Level-wise parallelism**: Components at same dependency level run concurrently
- **Group-wise load balancing**: Work-stealing within each level
- **Minimal synchronization**: Only at level boundaries
- **Deterministic execution**: Same results regardless of thread count
- **Scalable**: Performance improves with more threads up to dependency chain limit

## Implementation Steps
1. Remove Event trait and dynamic routing system
2. Create Signal type and ConnectionManager
3. Implement single-threaded CycleEngine with three-phase evaluation
4. Add topological level analysis to ConnectionManager
5. Implement ParallelCycleEngine with level-based partitioning
6. Add component grouping and load balancing
7. Update SimulationEngine to use parallel or sequential engine
8. Migrate components to new trait structure

## Benefits
- **Simplicity**: No complex event routing or subscriptions
- **Performance**: Direct connections, no dynamic dispatch
- **Type Safety**: Users control signal types at compile time
- **Hardware Accuracy**: Matches real circuit evaluation
- **Parallelization Ready**: Natural parallel execution model
- **Scalability**: Performance scales with available threads
- **User Control**: Framework provides structure, users define semantics