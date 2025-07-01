# Memory-Based Architecture Refactor Guide

This guide refactors the simulation engine to a simplified memory-based architecture:

1. **Processing components** are stateless and communicate through memory
2. **Memory components** hold all system state 
3. **Memory proxy** provides controlled access with proper cycle timing
4. **Previous-cycle reads** eliminate contention (flip-flop timing model)

---

## Core Architecture Principles

### Simplified Memory Access Model
- **Memory components** have one input port and one output port (for connection knowledge)
- **Engine-level memory proxy** provides centralized memory access control
- **Reads** return previous cycle's state (no contention)
- **Writes** are buffered and applied at cycle end
- **Components** access memory via engine proxy: `proxy.read(port, address)`

### Component Separation
- **Processing Components**: Pure combinational logic, no internal state, no proxy management
- **Memory Components**: Hold all system state (registers, FIFOs, caches, etc.)
- **Engine Memory Proxy**: Centralized interface controlling memory access timing

---

## Phase 1: Define New Component Architecture

### Step 1.1: Update Core Component Architecture

**Action:** Replace the entire content of `src/core/component.rs`:

```rust
use super::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

// Use existing ComponentValue for type consistency
pub type Event = ComponentValue;

// Engine-level memory proxy interface - centralized memory access
pub trait EngineMemoryProxy {
    fn read(&self, component_id: &ComponentId, port: &str, address: &str) -> Option<Event>;
    fn write(&mut self, component_id: &ComponentId, port: &str, address: &str, data: Event);
}

// Component behavior traits - define what component types can do
pub trait ProcessingBehavior {
    fn input_ports(&self) -> Vec<&'static str>;
    fn output_ports(&self) -> Vec<&'static str>;
    fn memory_ports(&self) -> Vec<&'static str> { vec![] }
    
    // Evaluate with access to engine memory proxy
    fn evaluate(&self, 
                inputs: &HashMap<String, Event>,
                memory_proxy: &mut dyn EngineMemoryProxy) -> HashMap<String, Event>;
}

pub trait MemoryBehavior {
    // Read from previous cycle's state snapshot
    fn read_snapshot(&self, address: &str) -> Option<Event>;
    
    // Write operation (applied immediately to current state)
    fn write(&mut self, address: &str, data: Event) -> bool;
    
    // Called at end of cycle to create snapshot for next cycle
    fn end_cycle(&mut self);
}

pub trait ProbeBehavior {
    fn probe(&mut self, source: &ComponentId, port: &str, event: &Event);
}

// Component instances - structs that can be instantiated multiple times
pub struct ProcessingComponent {
    pub id: ComponentId,
    pub behavior: Box<dyn ProcessingBehavior + Send>,
}

pub struct MemoryComponent {
    pub id: ComponentId,
    pub memory_id: String,
    pub behavior: Box<dyn MemoryBehavior + Send>,
}

pub struct ProbeComponent {
    pub id: ComponentId,
    pub behavior: Box<dyn ProbeBehavior + Send>,
}

impl ProcessingComponent {
    pub fn new(id: ComponentId, behavior: Box<dyn ProcessingBehavior + Send>) -> Self {
        Self { id, behavior }
    }
    
    pub fn component_id(&self) -> &ComponentId {
        &self.id
    }
}

impl MemoryComponent {
    pub fn new(id: ComponentId, memory_id: String, behavior: Box<dyn MemoryBehavior + Send>) -> Self {
        Self { id, memory_id, behavior }
    }
    
    pub fn component_id(&self) -> &ComponentId {
        &self.id
    }
    
    pub fn memory_id(&self) -> &str {
        &self.memory_id
    }
}

impl ProbeComponent {
    pub fn new(id: ComponentId, behavior: Box<dyn ProbeBehavior + Send>) -> Self {
        Self { id, behavior }
    }
    
    pub fn component_id(&self) -> &ComponentId {
        &self.id
    }
}
```

### Step 1.2: Create Cycle Engine

**Action:** Create new file `src/core/cycle_engine.rs`:

```rust
use super::component::{ProcessingComponent, MemoryComponent, ProbeComponent, EngineMemoryProxy, Event};
use super::types::ComponentId;
use std::collections::HashMap;

pub struct CycleEngine {
    processing_components: HashMap<ComponentId, ProcessingComponent>,
    memory_components: HashMap<ComponentId, MemoryComponent>,
    probe_components: HashMap<ComponentId, ProbeComponent>,
    
    // Memory connections: (component_id, port) -> memory_component_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,
    
    // Port connections: (source_id, port) -> Vec<(target_id, port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    
    // Store component outputs from previous cycle for current cycle inputs
    previous_cycle_outputs: HashMap<(ComponentId, String), Event>,
    
    current_cycle: u64,
}

// Engine's centralized memory proxy
pub struct CentralMemoryProxy<'a> {
    memory_components: &'a mut HashMap<ComponentId, MemoryComponent>,
    memory_connections: &'a HashMap<(ComponentId, String), ComponentId>,
}

impl<'a> EngineMemoryProxy for CentralMemoryProxy<'a> {
    fn read(&self, component_id: &ComponentId, port: &str, address: &str) -> Option<Event> {
        let mem_id = self.memory_connections.get(&(component_id.clone(), port.to_string()))?;
        self.memory_components.get(mem_id)?.behavior.read_snapshot(address)
    }
    
    fn write(&mut self, component_id: &ComponentId, port: &str, address: &str, data: Event) {
        if let Some(mem_id) = self.memory_connections.get(&(component_id.clone(), port.to_string())) {
            if let Some(memory) = self.memory_components.get_mut(mem_id) {
                memory.behavior.write(address, data);
            }
        }
    }
}

impl CycleEngine {
    pub fn new() -> Self {
        Self {
            processing_components: HashMap::new(),
            memory_components: HashMap::new(),
            probe_components: HashMap::new(),
            memory_connections: HashMap::new(),
            connections: HashMap::new(),
            previous_cycle_outputs: HashMap::new(),
            current_cycle: 0,
        }
    }
    
    pub fn register_processing(&mut self, component: ProcessingComponent) {
        let id = component.component_id().clone();
        self.processing_components.insert(id, component);
    }
    
    pub fn register_memory(&mut self, component: MemoryComponent) {
        let id = component.component_id().clone();
        self.memory_components.insert(id, component);
    }
    
    pub fn register_probe(&mut self, component: ProbeComponent) {
        let id = component.component_id().clone();
        self.probe_components.insert(id, component);
    }
    
    pub fn connect_memory(&mut self, proc_id: ComponentId, port: String, mem_id: ComponentId) {
        self.memory_connections.insert((proc_id, port), mem_id);
    }
    
    pub fn connect(&mut self, source: (ComponentId, String), target: (ComponentId, String)) {
        self.connections.entry(source).or_default().push(target);
    }
    
    pub fn run_cycle(&mut self) {
        // 1. Collect current cycle outputs
        let mut current_cycle_outputs: HashMap<(ComponentId, String), Event> = HashMap::new();
        
        // Create a separate scope for the memory proxy to avoid borrow conflicts
        {
            let mut proxy = CentralMemoryProxy {
                memory_components: &mut self.memory_components,
                memory_connections: &self.memory_connections,
            };
            
            // 2. Execute all processing components
            for (comp_id, component) in &self.processing_components {
                // Gather inputs for this component from PREVIOUS cycle outputs
                let mut inputs = HashMap::new();
                for input_port in component.behavior.input_ports() {
                    // Look for connections to this input port
                    for ((source_id, source_port), targets) in &self.connections {
                        for (target_id, target_port) in targets {
                            if target_id == comp_id && target_port == input_port {
                                // KEY FIX: Use previous_cycle_outputs instead of current cycle
                                if let Some(event) = self.previous_cycle_outputs.get(&(source_id.clone(), source_port.clone())) {
                                    inputs.insert(input_port.to_string(), event.clone());
                                }
                            }
                        }
                    }
                }
                
                // Evaluate component with memory proxy access
                let outputs = component.behavior.evaluate(&inputs, &mut proxy);
                
                // Store outputs for NEXT cycle
                for output_port in component.behavior.output_ports() {
                    if let Some(event) = outputs.get(output_port) {
                        current_cycle_outputs.insert((comp_id.clone(), output_port.to_string()), event.clone());
                    }
                }
            }
        }
        
        // 3. Trigger probes for current cycle outputs
        for ((source_id, source_port), event) in &current_cycle_outputs {
            for (probe_id, probe) in &mut self.probe_components {
                probe.behavior.probe(source_id, source_port, event);
            }
        }
        
        // 4. Update previous cycle outputs for next cycle
        self.previous_cycle_outputs = current_cycle_outputs;
        
        // 5. End cycle on all memory components (create next snapshot)
        for memory in self.memory_components.values_mut() {
            memory.behavior.end_cycle();
        }
        
        self.current_cycle += 1;
    }
    
    pub fn current_cycle(&self) -> u64 {
        self.current_cycle
    }
}
```

---

## Phase 2: Example Memory Component

### Step 2.1: Create FIFO Memory

**Action:** Create new file `src/core/memory_components/fifo.rs`:

```rust
use crate::core::component::{MemoryBehavior, Event, MemoryComponent};
use crate::core::types::{ComponentId, ComponentValue};
use std::collections::VecDeque;

pub struct FifoBehavior {
    capacity: usize,
    // Current state (written to during cycle)
    data: VecDeque<Event>,
    // Previous cycle snapshot (read from during cycle)
    snapshot: VecDeque<Event>,
}

impl FifoBehavior {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            data: VecDeque::new(),
            snapshot: VecDeque::new(),
        }
    }
}

impl MemoryBehavior for FifoBehavior {
    fn read_snapshot(&self, address: &str) -> Option<Event> {
        match address {
            "pop" => self.snapshot.front().cloned(),
            "can_read" => Some(ComponentValue::Bool(!self.snapshot.is_empty())),
            "length" => Some(ComponentValue::Int(self.snapshot.len() as i64)),
            _ => None,
        }
    }
    
    fn write(&mut self, address: &str, data: Event) -> bool {
        match address {
            "push" => {
                if self.data.len() < self.capacity {
                    self.data.push_back(data);
                    true
                } else {
                    false
                }
            }
            "pop" => {
                if !self.data.is_empty() {
                    self.data.pop_front();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    fn end_cycle(&mut self) {
        // Create snapshot for next cycle's reads
        self.snapshot = self.data.clone();
    }
}

// Helper function to create FIFO memory instances
pub fn create_fifo_memory(component_id: ComponentId, memory_id: String, capacity: usize) -> MemoryComponent {
    let behavior = Box::new(FifoBehavior::new(capacity));
    MemoryComponent::new(component_id, memory_id, behavior)
}
```

---

## Key Benefits

### 1. **Multiple Component Instances**
- **Same component type, different IDs**: Create `cpu1`, `cpu2`, `cpu3` from same `ProcessorBehavior`
- **Struct-based instances**: Components are concrete structs that can be instantiated multiple times
- **Behavior separation**: Component behavior (trait) separated from component instance (struct)

### 2. **Simplicity**
- Direct memory access via engine proxy: `proxy.read(component_id, port, address)`
- No complex REQ/ACK protocol or multiple proxy management
- Centralized memory access control in engine

### 3. **Correct Timing**
- Reads return previous cycle state (no contention)
- Writes are buffered until cycle end
- Maintains flip-flop timing model automatically

### 4. **Clean Architecture**
- Processing components stay truly stateless (no proxy management)
- Memory components handle all state with clear snapshot/write separation
- Engine provides centralized memory coordination

### 5. **Easy Implementation**
- Single engine-level proxy vs multiple proxy instances
- Clear connection mapping: `(component_id, port) -> memory_id`
- Components just call `proxy.read()`/`proxy.write()` during evaluation

---

## Implementation Plan

1. **Update component architecture** with struct-based instances and behavior traits
2. **Create cycle engine** with centralized memory proxy
3. **Implement example memory** components (FIFO, register) 
4. **Convert existing components** to use new struct + behavior pattern
5. **Update simulation engine** to use new cycle engine

## Usage Examples

### Creating Multiple Component Instances

```rust
// Create multiple processors of the same type
let cpu_behavior = Box::new(ProcessorBehavior::new());
let cpu1 = ProcessingComponent::new("cpu1".to_string(), cpu_behavior);

let cpu_behavior2 = Box::new(ProcessorBehavior::new()); 
let cpu2 = ProcessingComponent::new("cpu2".to_string(), cpu_behavior2);

// Create multiple FIFO memories
let fifo1 = create_fifo_memory("fifo1".to_string(), "mem1".to_string(), 16);
let fifo2 = create_fifo_memory("fifo2".to_string(), "mem2".to_string(), 32);

// Register all instances
engine.register_processing(cpu1);
engine.register_processing(cpu2);
engine.register_memory(fifo1);
engine.register_memory(fifo2);
```

This approach cleanly separates component type definition from component instances!

---

## Critical Logic Bugs Found in Implementation

### ✅ Bug 1: Borrow Checker Violation in Memory Proxy - FIXED

**Location:** `cycle_engine.rs:106-109`

**Problem:** The memory proxy held a mutable borrow of `memory_components` for the entire component evaluation loop, preventing components from accessing memory through the proxy.

**Fix Applied:** Implemented interior mutability using `RefCell`:
- **Changed:** `HashMap<ComponentId, Box<dyn MemoryComponent>>` → `HashMap<ComponentId, RefCell<Box<dyn MemoryComponent>>>`
- **Memory reads:** `memory_ref.borrow().read_snapshot(address)`
- **Memory writes:** `memory_ref.borrow_mut().write(address, data)`
- **Proxy creation:** No longer requires `&mut` reference
- **Result:** Components can now successfully access memory during evaluation

### ✅ Bug 2: Non-Deterministic Execution Order - FIXED

**Location:** `cycle_engine.rs:163-164`

**Problem:** Components were processed in HashMap key order, which was non-deterministic and made simulations non-reproducible.

**Fix Applied:** Implemented topological sorting using Kahn's algorithm:
- **Added field:** `execution_order: Vec<ComponentId>` for sorted execution order
- **Added method:** `build_execution_order()` with Kahn's algorithm implementation
- **Dependency analysis:** Builds adjacency list from component connections
- **Cycle detection:** Returns error if circular dependencies detected
- **Deterministic ordering:** Sorts components at each step for reproducible results
- **Integration:** SimulationEngine calls `build_execution_order()` after component registration
- **Result:** Components now execute in dependency-aware, deterministic order

### ✅ Bug 3: Input Port Collision Handling - FIXED

**Location:** `cycle_engine.rs:97-99`

**Problem:** Multiple sources to same input port silently overwrite each other:
```rust
inputs.insert(input_port.to_string(), event.clone());  // Overwrites previous value
```
Should detect and handle as error or combine values.

**Fix Applied:** Added validation in `connect()` method:
- **Detection logic:** Checks existing connections for same target input port
- **Error prevention:** Returns error when attempting multiple drivers to same input
- **Coverage:** Works for both processing and memory components
- **Result:** Multiple drivers now properly rejected with descriptive error messages

### ✅ Bug 4: Missing Connection Validation - FIXED

**Location:** `simulation_engine.rs:28-37`

**Problem:** ConnectionManager has proper validation, but CycleEngine accepts connections without validation. Invalid memory connections could cause runtime panics.

**Fix Applied:** Added comprehensive validation to both connection methods:
- **Component validation:** `connect()` and `connect_memory()` verify component existence
- **Port validation:** Validates source/target ports exist on specified components
- **Memory validation:** `connect_memory()` checks processing component memory ports
- **Error propagation:** SimulationEngine now handles validation errors during construction
- **Result:** Invalid connections caught at setup time, preventing runtime panics

### ✅ Bug 5: Incomplete Probe Integration - FIXED

**Location:** `cycle_engine.rs:330-343` (previously 292-297)

**Problem:** ConnectionManager tracked probe connections (`probes` field), but CycleEngine ignored this and probed all components instead of just connected ones.

**Fix Applied:** Implemented complete probe connection handling:
- **Added field:** `probes: HashMap<(ComponentId, String), Vec<ComponentId>>` to CycleEngine
- **Added method:** `connect_probe()` with comprehensive validation
- **Connection transfer:** SimulationEngine now transfers probe connections with validation
- **Selective triggering:** `run_cycle()` only triggers probes connected to specific source ports
- **Tests added:** `test_probe_connection_validation()` and `test_selective_probe_triggering()`
- **Result:** Probes now only receive events from their connected source ports

### ✅ Bug 6: Cycle 0 Cold Start Issue - NOT A PROBLEM

**Location:** `cycle_engine.rs:96-97`

**Problem:** All components get empty inputs on first cycle due to empty `previous_cycle_outputs`. This may cause initialization issues for components expecting valid inputs.

**Analysis:** This is not actually a bug - it's the correct behavior for the memory-based architecture:
- **Components should be self-starting:** Well-designed components generate their own initial events/outputs when needed
- **Memory components provide defaults:** Memory components can return default values when accessed
- **Realistic simulation:** Real hardware systems also start with undefined inputs and must bootstrap themselves
- **Design principle:** Components that require specific initial conditions should implement initialization logic internally

**Status:** No fix required - this is expected and correct behavior for a cycle-based simulation engine.

### ✅ Bug 7: Missing Error Propagation - FIXED

**Location:** Memory proxy interface

**Problem:** Memory operations can fail (e.g., invalid addresses), but the proxy interface doesn't propagate errors to components:
```rust
fn read(&self, component_id: &ComponentId, port: &str, address: &str) -> Option<Event>;
fn write(&mut self, component_id: &ComponentId, port: &str, address: &str, data: Event);
```

**Fix Applied:** Implemented Result-based error propagation:
- **Added `MemoryError` enum** with `InvalidAddress`, `InvalidPort`, `MemoryNotFound`, and `OperationFailed` variants
- **Updated interface:** `read()` returns `Result<Option<Event>, MemoryError>`, `write()` returns `Result<(), MemoryError>`
- **Enhanced validation:** CentralMemoryProxy now validates ports and memory existence with descriptive errors
- **Write failure detection:** Memory component write failures are now captured and propagated
- **Result:** Components can now detect and handle memory operation failures instead of experiencing silent failures
