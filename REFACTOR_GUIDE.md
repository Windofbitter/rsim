# LLM Coder's Guide: Refactoring to a Connection-Based Architecture

This guide provides step-by-step instructions to refactor the simulation engine from the old event-driven architecture to the new connection-based architecture, as detailed in `COMPONENT_CONNECTION_REFACTOR.md`.

Execute these steps sequentially.

---

## Phase 1: Core Framework Restructuring

Our first goal is to remove the old event-based modules and lay the foundation for the new component model.

### Step 1.1: File Cleanup

1.  **Delete** the following files, as they are being replaced entirely:
    *   `src/core/event.rs`
    *   `src/core/event_scheduler.rs`
2.  **Rename** the file `src/core/event_manager.rs` to `src/core/connection_manager.rs`. We will replace its contents later.

### Step 1.2: Define Core Traits and Types in `src/core/component.rs`

**Action:** Overwrite the entire content of `src/core/component.rs` with the new component trait definitions.

```rust
use super::types::ComponentId;
use std::any::Any;
use std::collections::HashMap;

// The universal message type passed between components.
pub type Signal = Box<dyn Any + Send>;

// The foundational trait for all components.
pub trait BaseComponent: Send {
    fn component_id(&self) -> &ComponentId;
}

// Trait for components that are part of the primary, active data-flow graph.
pub trait ActiveComponent: BaseComponent {
    fn input_ports(&self) -> Vec<&'static str>;
    fn output_port(&self) -> &'static str;
}

// Trait for stateless components that produce an output based on their current inputs.
pub trait CombinationalComponent: ActiveComponent {
    fn evaluate(&self, port_signals: &HashMap<String, Signal>) -> Option<Signal>;
}

// Trait for stateful components that have a clocked behavior.
pub trait SequentialComponent: ActiveComponent {
    fn current_output(&self) -> Option<Signal>;
    fn prepare_next_state(&mut self, port_signals: &HashMap<String, Signal>);
    fn commit_state_change(&mut self);
}

// Trait for passive monitoring components (e.g., metrics collectors, loggers).
pub trait ProbeComponent: BaseComponent {
    fn probe(&mut self, signal: &Signal);
}

// An enum to hold any type of component in the ConnectionManager.
pub enum Component {
    Combinational(Box<dyn CombinationalComponent>),
    Sequential(Box<dyn SequentialComponent>),
    Probe(Box<dyn ProbeComponent>),
}

// Add helper methods for easy access.
impl Component {
    pub fn as_base(&self) -> &dyn BaseComponent {
        match self {
            Component::Combinational(c) => c.as_ref(),
            Component::Sequential(c) => c.as_ref(),
            Component::Probe(c) => c.as_ref(),
        }
    }
}
```

---

## Phase 2: Implement the Simulation Machinery

Now we will implement the core logic that manages connections and executes simulation cycles.

### Step 2.1: Implement `ConnectionManager`

**Action:** Overwrite the entire content of `src/core/connection_manager.rs` with the following:

```rust
use super::component::{Component, ProbeComponent, Signal};
use super::types::ComponentId;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct ConnectionManager {
    pub components: HashMap<ComponentId, Component>,
    // Active data-flow connections: (Source Component, Source Port) -> Vec<(Target Component, Target Port)>
    pub connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    // Passive monitoring probes: (Source Component, Source Port) -> Vec<ProbeComponentId>
    pub probes: HashMap<(ComponentId, String), Vec<ComponentId>>,
    // The calculated, safe execution order for combinational components.
    pub combinational_order: Vec<ComponentId>,
    // The list of all sequential components.
    pub sequential_ids: Vec<ComponentId>,
    
    // NEW: Reverse mapping for efficient input gathering
    // Maps (target_component, input_port) -> (source_component, source_port)
    input_sources: HashMap<(ComponentId, String), (ComponentId, String)>,
    
    // NEW: Signal storage for current cycle
    current_signals: HashMap<(ComponentId, String), Signal>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            connections: HashMap::new(),
            probes: HashMap::new(),
            combinational_order: Vec::new(),
            sequential_ids: Vec::new(),
            input_sources: HashMap::new(),
            current_signals: HashMap::new(),
        }
    }

    pub fn register_component(&mut self, component: Component) {
        let id = component.as_base().component_id().clone();
        if let Component::Sequential(_) = &component {
            self.sequential_ids.push(id.clone());
        }
        self.components.insert(id, component);
    }

    pub fn connect(&mut self, source: (ComponentId, String), target: (ComponentId, String)) {
        self.connections.entry(source).or_default().push(target);
    }

    pub fn add_probe(&mut self, source_port: (ComponentId, String), probe_id: ComponentId) {
        self.probes.entry(source_port).or_default().push(probe_id);
    }

    /// Analyzes the graph of combinational components to find a safe execution order.
    /// This is a topological sort. It will return an error if a cycle is detected.
    pub fn build_evaluation_order(&mut self) -> Result<(), String> {
        let mut adj_list = HashMap::new();
        let mut in_degree = HashMap::new();
        let mut combinational_ids = HashSet::new();

        // Initialize graph data structures
        for (id, comp) in &self.components {
            if let Component::Combinational(_) = comp {
                combinational_ids.insert(id.clone());
                in_degree.entry(id.clone()).or_insert(0);
                adj_list.entry(id.clone()).or_insert_with(Vec::new);
            }
        }

        // Build adjacency list and in-degrees from connections
        for (source, targets) in &self.connections {
            let (source_id, _) = source;
            if !combinational_ids.contains(source_id) { continue; }

            for (target_id, _) in targets {
                if !combinational_ids.contains(target_id) { continue; }
                adj_list.get_mut(source_id).unwrap().push(target_id.clone());
                *in_degree.entry(target_id.clone()).or_insert(0) += 1;
            }
        }

        // Kahn's algorithm for topological sort
        let mut queue: VecDeque<ComponentId> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();
        
        let mut sorted_order = Vec::new();
        while let Some(u) = queue.pop_front() {
            sorted_order.push(u.clone());
            if let Some(neighbors) = adj_list.get(&u) {
                for v in neighbors {
                    if let Some(degree) = in_degree.get_mut(v) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(v.clone());
                        }
                    }
                }
            }
        }

        if sorted_order.len() == combinational_ids.len() {
            self.combinational_order = sorted_order;
            Ok(())
        } else {
            Err("Combinational cycle detected in component graph.".to_string())
        }
    }
    
    /// Builds reverse mapping for efficient input signal gathering
    pub fn build_input_mapping(&mut self) {
        self.input_sources.clear();
        
        for ((source_id, source_port), targets) in &self.connections {
            for (target_id, target_port) in targets {
                self.input_sources.insert(
                    (target_id.clone(), target_port.clone()),
                    (source_id.clone(), source_port.clone())
                );
            }
        }
    }
    
    /// Gathers input signals for a component based on its input ports
    pub fn gather_inputs(&self, component_id: &ComponentId, input_ports: &[&str]) 
        -> HashMap<String, Signal> {
        let mut inputs = HashMap::new();
        
        for port in input_ports {
            if let Some((source_id, source_port)) = 
                self.input_sources.get(&(component_id.clone(), port.to_string())) {
                
                if let Some(signal) = self.current_signals.get(&(source_id.clone(), source_port.clone())) {
                    // Clone the signal for this component
                    inputs.insert(port.to_string(), signal.clone());
                }
            }
        }
        
        inputs
    }
    
    /// Publishes a signal and triggers all associated probes
    pub fn publish_signal(&mut self, source: (ComponentId, String), signal: Signal) {
        // Store signal for input gathering
        self.current_signals.insert(source.clone(), signal.clone());
        
        // Trigger all probes for this output port
        if let Some(probe_ids) = self.probes.get(&source) {
            for probe_id in probe_ids {
                if let Some(Component::Probe(probe)) = self.components.get_mut(probe_id) {
                    probe.probe(&signal);
                }
            }
        }
    }
}
```

### Step 2.2: Create the `CycleEngine`

**Action:** Create a new file `src/core/cycle_engine.rs` and add the following content. This will be our single-threaded, synchronous simulation engine.

```rust
use super::component::{Component, Signal};
use super::connection_manager::ConnectionManager;
use std::collections::HashMap;

pub struct CycleEngine {
    pub connection_manager: ConnectionManager,
    pub current_cycle: u64,
}

impl CycleEngine {
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
            current_cycle: 0,
        }
    }

    pub fn run_cycle(&mut self) {
        // Clear previous cycle's signals
        self.connection_manager.current_signals.clear();
        
        // Phase 1: Combinational Propagation
        // Evaluate all combinational components in the pre-calculated topological order.
        for comp_id in &self.connection_manager.combinational_order.clone() {
            if let Some(Component::Combinational(comp)) = 
                self.connection_manager.components.get(comp_id) {
                
                // Gather inputs using the reverse mapping
                let inputs = self.connection_manager.gather_inputs(
                    comp_id, 
                    &comp.input_ports()
                );
                
                // Evaluate component
                if let Some(output_signal) = comp.evaluate(&inputs) {
                    // Publish signal (stores + triggers probes)
                    let output_port = (comp_id.clone(), comp.output_port().to_string());
                    self.connection_manager.publish_signal(output_port, output_signal);
                }
            }
        }

        // Phase 2: Sequential State Preparation
        // All sequential components read their inputs and prepare their next state.
        for comp_id in &self.connection_manager.sequential_ids.clone() {
            if let Some(Component::Sequential(comp)) = 
                self.connection_manager.components.get_mut(comp_id) {
                
                let inputs = self.connection_manager.gather_inputs(
                    comp_id, 
                    &comp.input_ports()
                );
                
                comp.prepare_next_state(&inputs);
            }
        }

        // Phase 3: Sequential State Commit + Output
        // All sequential components atomically update their state for the next cycle.
        for comp_id in &self.connection_manager.sequential_ids.clone() {
            if let Some(Component::Sequential(comp)) = 
                self.connection_manager.components.get_mut(comp_id) {
                
                comp.commit_state_change();
                
                // Publish sequential component's output
                if let Some(output_signal) = comp.current_output() {
                    let output_port = (comp_id.clone(), comp.output_port().to_string());
                    self.connection_manager.publish_signal(output_port, output_signal);
                }
            }
        }

        self.current_cycle += 1;
    }
}
```
*Note: The `run_cycle` implementation now includes complete signal routing through the reverse mapping and probe triggering via the `publish_signal` method.*

### Step 2.3: Update `SimulationEngine`

**Action:** Modify `src/core/simulation_engine.rs` to use the new `CycleEngine`.

```rust
// Remove old imports for EventManager and EventScheduler
use super::component::Component;
use super::connection_manager::ConnectionManager;
use super::cycle_engine::CycleEngine;

pub struct SimulationEngine {
    cycle_engine: CycleEngine,
    max_cycles: Option<u64>,
}

impl SimulationEngine {
    pub fn new(connection_manager: ConnectionManager, max_cycles: Option<u64>) -> Result<Self, String> {
        let mut engine = Self {
            cycle_engine: CycleEngine::new(connection_manager),
            max_cycles,
        };
        
        // Build the required mappings before simulation can run
        engine.cycle_engine.connection_manager.build_evaluation_order()?;
        engine.cycle_engine.connection_manager.build_input_mapping();
        
        Ok(engine)
    }

    pub fn run(&mut self) -> Result<u64, String> {
        while self.max_cycles.map_or(true, |max| self.current_cycle() < max) {
            self.step()?;
        }
        Ok(self.current_cycle())
    }

    pub fn step(&mut self) -> Result<bool, String> {
        self.cycle_engine.run_cycle();
        Ok(true)
    }

    pub fn current_cycle(&self) -> u64 {
        self.cycle_engine.current_cycle
    }
}
```

---

## Phase 3: Update Module Definitions

**Action:** Overwrite `src/core/mod.rs` to export the new modules.

```rust
pub mod component;
pub mod connection_manager;
pub mod cycle_engine;
pub mod simulation_engine;
pub mod types;
```

---

## Summary

This refactor guide transforms the simulation engine from event-driven messaging to direct port-to-port connections with passive monitoring probes. Here's what each step accomplishes:

### **Phase 1: Core Framework Restructuring**
- **Step 1.1**: Remove old event system files (`event.rs`, `event_scheduler.rs`) and rename `event_manager.rs` to `connection_manager.rs`
- **Step 1.2**: Define new component trait hierarchy:
  - `BaseComponent`: Foundation for all components
  - `ActiveComponent`: Components in the main data flow with input/output ports
  - `CombinationalComponent`: Stateless components that evaluate inputs immediately
  - `SequentialComponent`: Stateful components with clocked behavior (prepare → commit)
  - `ProbeComponent`: Passive monitoring components for metrics/logging
  - `Component` enum: Storage wrapper for all component types

### **Phase 2: Simulation Machinery**
- **Step 2.1**: Implement `ConnectionManager` with:
  - Component registration and connection management
  - Topological sort for combinational component ordering (prevents cycles)
  - **Reverse mapping system**: Efficiently maps component inputs to their signal sources
  - **Signal routing**: `gather_inputs()` collects signals for component evaluation
  - **Probe triggering**: `publish_signal()` stores signals AND triggers all attached probes
- **Step 2.2**: Create `CycleEngine` with three-phase execution:
  - **Phase 1**: Combinational propagation in dependency order
  - **Phase 2**: Sequential state preparation (read inputs, calculate next state)  
  - **Phase 3**: Sequential state commit + output (atomic state update)
- **Step 2.3**: Update `SimulationEngine` to use `CycleEngine` and automatically build required mappings

### **Phase 3: Module Integration**
- Update `mod.rs` to export the new architecture

### **Key Architectural Benefits**
- **Performance**: Direct connections eliminate event routing overhead
- **Determinism**: Topological ordering ensures consistent execution
- **Monitoring**: Probes provide passive observation without affecting core logic
- **Hardware Accuracy**: Three-phase sequential evaluation matches real circuit behavior
- **Maintainability**: Clear separation between active data flow and passive monitoring

This guide provides the complete blueprint for the core refactoring. The next major step, after this foundation is built and compiling, would be to migrate the components in `examples/burger_production/components/` to use the new `CombinationalComponent` and `SequentialComponent` traits.
