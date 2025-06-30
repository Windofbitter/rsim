# Component Connection Refactoring Plan

## Overview
Refactor to port-to-port connections with separate combinational and sequential component types.

## Component Model

### Base Component Trait
```rust
pub trait BaseComponent: Send {
    fn component_id(&self) -> &ComponentId;
    fn input_ports(&self) -> Vec<&'static str>;
    fn output_port(&self) -> &'static str;
    fn output_event_type(&self) -> &'static str;
}
```

### Specialized Component Traits
```rust
pub trait CombinationalComponent: BaseComponent {
    fn evaluate(&self, port_events: HashMap<String, Box<dyn Event>>) 
        -> Option<Box<dyn Event>>;
}

pub trait SequentialComponent: BaseComponent {
    fn current_output(&self) -> Option<Box<dyn Event>>;
    fn prepare_next_state(&mut self, port_events: &HashMap<String, Box<dyn Event>>);
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

## Connection Management
```rust
pub struct ConnectionManager {
    components: HashMap<ComponentId, Component>,
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    combinational_order: Vec<ComponentId>,  // Topologically sorted
    sequential_ids: Vec<ComponentId>,
}
```

## Cycle Evaluation
```rust
impl CycleEngine {
    fn run_cycle(&mut self) {
        // Phase 1: Combinational propagation (single pass)
        for comp_id in &self.combinational_order {
            // Evaluate and route outputs
        }
        
        // Phase 2: Sequential state preparation
        for comp_id in &self.sequential_ids {
            // Prepare next state based on inputs
        }
        
        // Phase 3: Sequential state commit (atomic)
        for comp_id in &self.sequential_ids {
            // Update all states simultaneously
        }
    }
}
```

## Implementation Steps
1. Create base and specialized component traits
2. Update ConnectionManager with Component enum
3. Implement topological sorting for combinational components
4. Create CycleEngine with three-phase evaluation
5. Update existing components to new trait structure

## Benefits
- Type-safe component handling
- Single-pass combinational evaluation
- Atomic sequential state updates
- Hardware-accurate timing model
- Parallelization ready