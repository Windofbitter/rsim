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
- **Memory proxy** provides read/write interface to connected components
- **Reads** return previous cycle's state (no contention)
- **Writes** are buffered and applied at cycle end
- **Components** access memory via `get_memory_by_port()`

### Component Separation
- **Processing Components**: Pure combinational logic, no internal state
- **Memory Components**: Hold all system state (registers, FIFOs, caches, etc.)
- **Memory Proxy**: Simple interface controlling when state changes occur

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

// Simple memory proxy interface - controls read/write timing
pub trait MemoryProxy {
    fn read(&self, address: &str) -> Option<Event>;  // Returns previous cycle state
    fn write(&mut self, address: &str, data: Event); // Buffered until cycle end
}

// Stateless processing components
pub trait ProcessingComponent: BaseComponent {
    fn input_ports(&self) -> Vec<&'static str>;
    fn output_ports(&self) -> Vec<&'static str>;
    
    // Get memory proxy for connected memory component
    fn get_memory_by_port(&mut self, port: &str) -> Option<&mut dyn MemoryProxy>;
    
    // Evaluate based on inputs (memory accessed via proxy during evaluation)
    fn evaluate(&mut self, inputs: &HashMap<String, Event>) -> HashMap<String, Event>;
}

// Stateful memory components
pub trait MemoryComponent: BaseComponent {
    fn memory_id(&self) -> &str;
    fn input_port(&self) -> &'static str { "in" }
    fn output_port(&self) -> &'static str { "out" }
    
    // Create proxy for this memory component
    fn create_proxy(&mut self) -> Box<dyn MemoryProxy>;
    
    // Called at end of cycle to apply buffered writes
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
use super::component::{ProcessingComponent, MemoryComponent, ProbeComponent};
use super::types::ComponentId;
use std::collections::HashMap;

pub struct CycleEngine {
    processing_components: HashMap<ComponentId, Box<dyn ProcessingComponent>>,
    memory_components: HashMap<ComponentId, Box<dyn MemoryComponent>>,
    probe_components: HashMap<ComponentId, Box<dyn ProbeComponent>>,
    
    // Memory connections: processing_component_id -> (port, memory_component_id)
    memory_connections: HashMap<ComponentId, HashMap<String, ComponentId>>,
    
    // Port connections: (source_id, port) -> Vec<(target_id, port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    
    current_cycle: u64,
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
        self.memory_connections.entry(proc_id).or_default().insert(port, mem_id);
    }
    
    pub fn connect(&mut self, source: (ComponentId, String), target: (ComponentId, String)) {
        self.connections.entry(source).or_default().push(target);
    }
    
    pub fn run_cycle(&mut self) {
        // 1. Create memory proxies for processing components
        // 2. Execute all processing components 
        // 3. Route outputs to connected components
        // 4. End cycle on all memory components (apply writes)
        
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
use crate::core::component::{MemoryComponent, MemoryProxy, Event, BaseComponent};
use crate::core::types::ComponentId;
use std::collections::VecDeque;
use std::sync::Arc;

pub struct FifoMemory {
    component_id: ComponentId,
    memory_id: String,
    capacity: usize,
    data: VecDeque<Event>,
    // Previous cycle snapshot for reads
    prev_data: VecDeque<Event>,
}

impl FifoMemory {
    pub fn new(component_id: ComponentId, memory_id: String, capacity: usize) -> Self {
        Self {
            component_id,
            memory_id,
            capacity,
            data: VecDeque::new(),
            prev_data: VecDeque::new(),
        }
    }
}

pub struct FifoProxy {
    prev_data: VecDeque<Event>,
    write_buffer: VecDeque<Event>,
    capacity: usize,
}

impl MemoryProxy for FifoProxy {
    fn read(&self, address: &str) -> Option<Event> {
        match address {
            "pop" => self.prev_data.front().cloned(),
            _ => None,
        }
    }
    
    fn write(&mut self, address: &str, data: Event) {
        match address {
            "push" => {
                if self.write_buffer.len() < self.capacity {
                    self.write_buffer.push_back(data);
                }
            }
            _ => {}
        }
    }
}

impl MemoryComponent for FifoMemory {
    fn memory_id(&self) -> &str {
        &self.memory_id
    }
    
    fn create_proxy(&mut self) -> Box<dyn MemoryProxy> {
        Box::new(FifoProxy {
            prev_data: self.prev_data.clone(),
            write_buffer: VecDeque::new(),
            capacity: self.capacity,
        })
    }
    
    fn end_cycle(&mut self) {
        self.prev_data = self.data.clone();
        // Apply writes from proxy would need to be implemented
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
- Direct memory access via simple read/write interface
- No complex REQ/ACK protocol to implement
- Components get memory via `get_memory_by_port()`

### 2. **Correct Timing**
- Reads return previous cycle state (no contention)
- Writes are buffered until cycle end
- Maintains flip-flop timing model automatically

### 3. **Clean Architecture**
- Processing components stay stateless
- Memory components handle all state
- Simple proxy controls when changes occur

### 4. **Easy Implementation**
- Much simpler than original guide's approach
- Fewer abstractions and interfaces
- Direct and intuitive API

---

## Implementation Plan

1. **Update component traits** with simplified interfaces
2. **Create cycle engine** with memory proxy management  
3. **Implement example memory** components (FIFO, register)
4. **Convert existing components** to use memory proxy pattern
5. **Update simulation engine** to use new cycle engine

This approach achieves the same architectural goals with significantly less complexity!
