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
use super::types::ComponentId;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

// Universal data type for inter-component communication
pub type Event = Arc<dyn Any + Send + Sync>;

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
    
    pub fn connect_memory(&mut self, proc_id: ComponentId, port: String, mem_id: ComponentId) {
        self.memory_connections.insert((proc_id, port), mem_id);
    }
    
    pub fn connect(&mut self, source: (ComponentId, String), target: (ComponentId, String)) {
        self.connections.entry(source).or_default().push(target);
    }
    
    pub fn run_cycle(&mut self) {
        // 1. Execute all processing components with centralized memory proxy
        let mut proxy = CentralMemoryProxy {
            memory_components: &mut self.memory_components,
            memory_connections: &self.memory_connections,
        };
        
        // 2. Collect component inputs and execute
        // 3. Route outputs to connected components
        // 4. End cycle on all memory components (create next snapshot)
        
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
use std::sync::Arc;

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
        match address {
            "pop" => self.snapshot.front().cloned(),
            "can_read" => Some(Arc::new(!self.snapshot.is_empty())),
            "length" => Some(Arc::new(self.snapshot.len())),
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
