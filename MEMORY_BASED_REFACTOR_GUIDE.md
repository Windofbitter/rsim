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

### Step 1.1: Update Core Component Traits

**Action:** Replace the entire content of `src/core/component.rs`:

```rust
use super::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

// Use existing ComponentValue for type consistency
pub type Event = ComponentValue;

// Base trait for all components
pub trait BaseComponent: Send {
    fn component_id(&self) -> &ComponentId;
}

// Engine-level memory proxy interface - centralized memory access
pub trait EngineMemoryProxy {
    fn read(&self, component_id: &ComponentId, port: &str, address: &str) -> Option<Event>;
    fn write(&mut self, component_id: &ComponentId, port: &str, address: &str, data: Event);
}

// Stateless processing components
pub trait ProcessingComponent: BaseComponent {
    fn input_ports(&self) -> Vec<&'static str>;
    fn output_ports(&self) -> Vec<&'static str>;
    fn memory_ports(&self) -> Vec<&'static str> { vec![] }
    
    // Evaluate with access to engine memory proxy
    fn evaluate(&self, 
                inputs: &HashMap<String, Event>,
                memory_proxy: &mut dyn EngineMemoryProxy) -> HashMap<String, Event>;
}

// Stateful memory components
pub trait MemoryComponent: BaseComponent {
    fn memory_id(&self) -> &str;
    fn input_port(&self) -> &'static str { "in" }
    fn output_port(&self) -> &'static str { "out" }
    
    // Read from previous cycle's state snapshot
    fn read_snapshot(&self, address: &str) -> Option<Event>;
    
    // Write operation (applied immediately to current state)
    fn write(&mut self, address: &str, data: Event) -> bool;
    
    // Called at end of cycle to create snapshot for next cycle
    fn end_cycle(&mut self);
}

// Passive monitoring components
pub trait ProbeComponent: BaseComponent {
    fn probe(&mut self, source: &ComponentId, port: &str, event: &Event);
}
```

### Step 1.2: Create Cycle Engine

**Action:** Create new file `src/core/cycle_engine.rs`:

```rust
use super::component::{ProcessingComponent, MemoryComponent, ProbeComponent, EngineMemoryProxy, Event};
use super::types::ComponentId;
use std::collections::HashMap;

pub struct CycleEngine {
    processing_components: HashMap<ComponentId, Box<dyn ProcessingComponent>>,
    memory_components: HashMap<ComponentId, Box<dyn MemoryComponent>>,
    probe_components: HashMap<ComponentId, Box<dyn ProbeComponent>>,
    
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
    memory_components: &'a mut HashMap<ComponentId, Box<dyn MemoryComponent>>,
    memory_connections: &'a HashMap<(ComponentId, String), ComponentId>,
}

impl<'a> EngineMemoryProxy for CentralMemoryProxy<'a> {
    fn read(&self, component_id: &ComponentId, port: &str, address: &str) -> Option<Event> {
        let mem_id = self.memory_connections.get(&(component_id.clone(), port.to_string()))?;
        self.memory_components.get(mem_id)?.read_snapshot(address)
    }
    
    fn write(&mut self, component_id: &ComponentId, port: &str, address: &str, data: Event) {
        if let Some(mem_id) = self.memory_connections.get(&(component_id.clone(), port.to_string())) {
            if let Some(memory) = self.memory_components.get_mut(mem_id) {
                memory.write(address, data);
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
    
    pub fn register_processing(&mut self, component: Box<dyn ProcessingComponent>) {
        let id = component.component_id().clone();
        self.processing_components.insert(id, component);
    }
    
    pub fn register_memory(&mut self, component: Box<dyn MemoryComponent>) {
        let id = component.component_id().clone();
        self.memory_components.insert(id, component);
    }
    
    pub fn register_probe(&mut self, component: Box<dyn ProbeComponent>) {
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
                for input_port in component.input_ports() {
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
                let outputs = component.evaluate(&inputs, &mut proxy);
                
                // Store outputs for NEXT cycle
                for output_port in component.output_ports() {
                    if let Some(event) = outputs.get(output_port) {
                        current_cycle_outputs.insert((comp_id.clone(), output_port.to_string()), event.clone());
                    }
                }
            }
        }
        
        // 3. Trigger probes for current cycle outputs
        for ((source_id, source_port), event) in &current_cycle_outputs {
            for (probe_id, probe) in &mut self.probe_components {
                probe.probe(source_id, source_port, event);
            }
        }
        
        // 4. Update previous cycle outputs for next cycle
        self.previous_cycle_outputs = current_cycle_outputs;
        
        // 5. End cycle on all memory components (create next snapshot)
        for memory in self.memory_components.values_mut() {
            memory.end_cycle();
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
use crate::core::component::{MemoryComponent, Event, BaseComponent};
use crate::core::types::ComponentId;
use std::collections::VecDeque;

pub struct FifoMemory {
    component_id: ComponentId,
    memory_id: String,
    capacity: usize,
    // Current state (written to during cycle)
    data: VecDeque<Event>,
    // Previous cycle snapshot (read from during cycle)
    snapshot: VecDeque<Event>,
}

impl FifoMemory {
    pub fn new(component_id: ComponentId, memory_id: String, capacity: usize) -> Self {
        Self {
            component_id,
            memory_id,
            capacity,
            data: VecDeque::new(),
            snapshot: VecDeque::new(),
        }
    }
}

impl MemoryComponent for FifoMemory {
    fn memory_id(&self) -> &str {
        &self.memory_id
    }
    
    fn read_snapshot(&self, address: &str) -> Option<Event> {
        use crate::core::types::ComponentValue;
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

impl BaseComponent for FifoMemory {
    fn component_id(&self) -> &ComponentId {
        &self.component_id
    }
}
```

---

## Key Benefits

### 1. **Simplicity**
- Direct memory access via engine proxy: `proxy.read(component_id, port, address)`
- No complex REQ/ACK protocol or multiple proxy management
- Centralized memory access control in engine

### 2. **Correct Timing**
- Reads return previous cycle state (no contention)
- Writes are buffered until cycle end
- Maintains flip-flop timing model automatically

### 3. **Clean Architecture**
- Processing components stay truly stateless (no proxy management)
- Memory components handle all state with clear snapshot/write separation
- Engine provides centralized memory coordination

### 4. **Easy Implementation**
- Single engine-level proxy vs multiple proxy instances
- Clear connection mapping: `(component_id, port) -> memory_id`
- Components just call `proxy.read()`/`proxy.write()` during evaluation

---

## Implementation Plan

1. **Update component traits** with engine-level proxy interfaces
2. **Create cycle engine** with centralized memory proxy
3. **Implement example memory** components (FIFO, register)
4. **Convert existing components** to use engine proxy pattern
5. **Update simulation engine** to use new cycle engine

This centralized approach is much cleaner than distributed proxy management!

---

## Critical Logic Bugs Found in Implementation

### ✅ Bug 1: Circular Dependency in Component Execution - FIXED

**Location:** `cycle_engine.rs:86-112`

**Problem:** Components try to gather inputs from other components' outputs in the same cycle, but outputs are only stored AFTER all components have been evaluated. This creates a circular dependency where:

1. Component A needs Component B's output as input
2. Component B's output isn't available until Component B evaluates  
3. Component B can't evaluate until all components have gathered inputs
4. Result: All components get empty inputs (except from memory)

**Fix Applied:** Added previous cycle output storage with proper timing:
- **Added field:** `previous_cycle_outputs: HashMap<(ComponentId, String), Event>`
- **Input gathering:** Uses previous cycle's outputs as current cycle's inputs
- **Output storage:** Current cycle outputs stored for next cycle's inputs
- **Hardware timing:** Matches flip-flop behavior where inputs come from previous clock cycle

### Bug 2: Borrow Conflict in Memory Proxy Scope

**Location:** `cycle_engine.rs:80-113`

**Problem:** The memory proxy holds a mutable borrow of `memory_components` for the entire component evaluation loop. This prevents components from actually accessing memory through the proxy because:

1. Proxy creation takes `&mut memory_components` 
2. Component evaluation happens inside this borrow scope
3. Any memory access attempts would require additional borrows of already-borrowed data
4. Rust prevents this at compile time

**Fix Required:** Restructure to separate memory proxy creation from component evaluation, or use interior mutability patterns.

### ✅ Bug 3: Missing Component State Persistence - FIXED

**Problem:** The current implementation doesn't store component outputs between cycles. Components can only access:
- Memory snapshots (previous cycle)
- Empty component inputs (due to Bug 1)

**Fix Applied:** Added cycle-to-cycle component output storage in Bug 1 fix:
- **Field added:** `previous_cycle_outputs: HashMap<(ComponentId, String), Event>`
- **Storage:** Current cycle outputs are stored at cycle end for next cycle
- **Access:** Components can now receive inputs from other components

### ✅ Bug 4: Incomplete Input Routing Logic - FIXED

**Location:** `cycle_engine.rs:89-101`

**Problem:** Input gathering logic assumes outputs exist in `cycle_outputs` during the same cycle, but `cycle_outputs` is empty at the start of each cycle.

**Fix Applied:** Updated input routing to use previous cycle outputs:
- **Changed:** Input gathering now uses `self.previous_cycle_outputs`
- **Timing:** Previous cycle's outputs become current cycle's inputs
- **Result:** Components can now properly receive data from connected components
