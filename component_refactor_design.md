# Component Architecture Refactor Design

## Current Issues
- Trait objects (`Box<dyn Component>`) create runtime overhead
- Cannot create multiple instances of same component type with different IDs
- Complex registration with separate methods per component type
- Type erasure makes debugging difficult

## Proposed Solution: Module-to-Component Pattern

### Core Concept
Replace trait-based components with:
1. **Modules** - Templates defining component behavior
2. **Component Manager** - Factory for creating instances from modules
3. **Struct-based Components** - Concrete instances with unique IDs
4. **Type-Safe Memory** - Memory components store domain-appropriate data structures (lists, tables, queues) instead of address-based storage

### Architecture Overview

```rust
// Module templates (behavior definitions)
pub enum ComponentModule {
    Processing(ProcessorModule),
    Memory(Box<dyn MemoryModuleTrait>),
    Probe(ProbeModule),
}

// Concrete component instances
pub struct Component {
    pub id: ComponentId,
    pub module_name: String,
    pub component_type: ComponentType,
    pub state: Box<dyn ComponentState>,
}

// Factory for creating instances
pub struct ComponentManager {
    registered_modules: HashMap<String, ComponentModule>,
    next_instance_id: u64,
}
```

### Key Types

#### ProcessorModule
```rust
pub struct PortSpec {
    pub name: &'static str,
    pub required: bool,
    pub description: Option<&'static str>,
}

pub struct ProcessorModule {
    pub name: String,
    pub input_ports: Vec<PortSpec>,
    pub output_ports: Vec<PortSpec>,
    pub memory_ports: Vec<&'static str>,
    pub evaluate_fn: fn(&mut EvaluationContext),
}

/// Context object passed to the evaluate_fn.
/// This is more performant than returning a HashMap, as it avoids allocations on every call.
/// It also provides an extensible API for adding new engine capabilities.
pub struct EvaluationContext<'a> {
    inputs: &'a HashMap<String, Event>,
    memory: &'a mut dyn EngineMemoryProxy,
    outputs: HashMap<String, Event>,
}

impl<'a> EvaluationContext<'a> {
    /// Get an input event from a named port.
    pub fn get_input(&self, port_name: &str) -> Option<&Event> {
        self.inputs.get(port_name)
    }

    /// Emit an output event to a named port.
    pub fn emit(&mut self, port_name: String, event: Event) {
        self.outputs.insert(port_name, event);
    }
    
    /// Get a mutable reference to the memory proxy to interact with memory components.
    pub fn memory(&mut self) -> &mut dyn EngineMemoryProxy {
        self.memory
    }
}

// Convenience constructor for simple ports
impl PortSpec {
    pub fn required(name: &'static str) -> Self {
        Self { name, required: true, description: None }
    }
    
    pub fn optional(name: &'static str) -> Self {
        Self { name, required: false, description: None }
    }
    
    pub fn with_description(name: &'static str, required: bool, description: &'static str) -> Self {
        Self { name, required, description: Some(description) }
    }
}
```

#### ProbeModule
Probes are components designed for observability, allowing for data collection, logging, and metrics aggregation without altering simulation logic. They typically have inputs but no outputs.

```rust
pub type ProbeFn = fn(state: &mut dyn ComponentState, event: &Event, timestamp: u64);

pub struct ProbeModule {
    pub name: String,
    /// A function to create the initial state for a new probe instance.
    pub create_state_fn: fn() -> Box<dyn ComponentState>,
    /// The function that will be called for each event the probe receives.
    pub probe_fn: ProbeFn,
}
```

#### MemoryModule with Type Erasure

**Rationale**: To allow the simulation engine to support any user-defined memory component, we use a trait object (`Box<dyn MemoryModuleTrait>`). This allows the `ComponentManager` to store a collection of different memory module types. While this uses dynamic dispatch, it provides critical extensibility: users of the engine can create their own custom memory types (e.g., for priority queues, caches, etc.) without having to modify the core engine code. This makes the system open to extension.

```rust
// Type-erased trait for memory modules
pub trait MemoryModuleTrait: Send + Sync {
    fn name(&self) -> &str;
    fn create_instance(&self, id: ComponentId) -> Box<dyn ComponentState>;
    fn type_name(&self) -> &'static str; // For debugging and validation
}

// Generic implementation for concrete memory types
pub struct MemoryModule<T: MemoryData> {
    pub name: String,
    pub initial_data: T,
    pub create_fn: fn(T) -> Box<dyn ComponentState>,
}

impl<T: MemoryData> MemoryModuleTrait for MemoryModule<T> {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn create_instance(&self, _id: ComponentId) -> Box<dyn ComponentState> {
        (self.create_fn)(self.initial_data.clone())
    }
    
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}
```

#### ComponentState and MemoryData
The `ComponentState` trait is the core of state management. It provides a unified interface for the engine to manage type-safe access to the underlying data.

**Rationale for `as_any`/`as_any_mut`**: Because the engine stores all component states as generic `Box<dyn ComponentState>` trait objects, it loses the concrete type information. These two methods provide a safe, standard mechanism to "downcast" the trait object back to its original concrete type (e.g., `QueueState`) at runtime, which is essential for the component's logic to operate on its data.

```rust
// The core trait for all component state.
// Developers can use a helper struct or a derive macro to implement this automatically.
pub trait ComponentState: Any + Send + Sync {
    /// Provides a reference to the state as a `std::any::Any` for downcasting.
    fn as_any(&self) -> &dyn Any;
    /// Provides a mutable reference to the state as a `std::any::Any` for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// A marker trait for a ComponentState that can be used as a memory component's data.
pub trait MemoryData: ComponentState + std::fmt::Debug + Clone {}

// Memory access through the proxy
impl EngineMemoryProxy {
    // Type-safe read with runtime checking
    pub fn read<T: MemoryData>(&self, component_id: &ComponentId) -> Result<&T, MemoryError> {
        let state = self.get_memory_state(component_id)?;
        // Downcasting via ComponentState's as_any method
        state.as_any()
            .downcast_ref::<T>()
            .ok_or_else(|| MemoryError::TypeMismatch)
    }
}
```

#### Component Manager API
```rust
impl ComponentManager {
    // Register behavior template
    pub fn register_module(&mut self, module: ComponentModule) -> Result<(), String>
    
    // Create instance from template
    pub fn create_component(&mut self, module_name: &str, instance_id: Option<ComponentId>) -> Result<Component, String>
    
    // Auto-generate unique IDs
    pub fn create_component_auto_id(&mut self, module_name: &str) -> Result<Component, String>
}
```

### Usage Example

```rust
// 1. Register processor module template
let adder_module = ProcessorModule {
    name: "adder".to_string(),
    input_ports: vec![
        PortSpec::required("a"),
        PortSpec::required("b"),
    ],
    output_ports: vec![
        PortSpec::with_description("sum", true, "Sum of inputs a and b"),
    ],
    memory_ports: vec![],
    evaluate_fn: |ctx| {
        // Components handle type flexibility internally
        let a = ctx.get_input("a")
            .and_then(|v| v.as_int().or_else(|| v.as_float().map(|f| f as i64)))
            .unwrap_or(0);
        let b = ctx.get_input("b")
            .and_then(|v| v.as_int().or_else(|| v.as_float().map(|f| f as i64)))
            .unwrap_or(0);
        ctx.emit("sum".to_string(), ComponentValue::Int(a + b));
    },
};
component_manager.register_module(ComponentModule::Processing(adder_module))?;

// 2. Register memory modules for different data types
// Queue memory for buffering requests
let queue_module = MemoryModule {
    name: "request_queue".to_string(),
    initial_data: VecDeque::<Request>::new(),
    create_fn: |data| Box::new(QueueState { queue: data }),
};
component_manager.register_module(ComponentModule::Memory(Box::new(queue_module)))?;

// Routing table memory
let routing_module = MemoryModule {
    name: "routing_table".to_string(),
    initial_data: vec![
        RouteEntry { dest: "10.0.0.0/8", next_hop: "192.168.1.1" },
        RouteEntry { dest: "172.16.0.0/12", next_hop: "192.168.1.2" },
    ],
    create_fn: |data| Box::new(TableState { entries: data }),
};
component_manager.register_module(ComponentModule::Memory(Box::new(routing_module)))?;

// 3. Create multiple instances
let adder1 = component_manager.create_component("adder", Some("adder_cpu1".to_string()))?;
let adder2 = component_manager.create_component("adder", Some("adder_cpu2".to_string()))?;
let queue1 = component_manager.create_component("request_queue", Some("input_buffer".to_string()))?;
let routing1 = component_manager.create_component("routing_table", Some("main_router".to_string()))?;
```

### Benefits

1. **Performance**: Direct struct access, no trait object overhead
2. **Multiple Instances**: Easy creation of many instances from same template
3. **Unified API**: Single registration and creation path
4. **Better Debugging**: Concrete types, no type erasure
5. **Extensibility**: Easy to add new component types through modules
6. **Memory Efficiency**: Shared behavior definitions, unique instance state
7. **Flexible Memory Data**: Store any Rust type (Vec, HashMap, custom structs) without artificial addressing
8. **Flexible Port Interface**: Required/optional ports with documentation, no rigid type constraints

### Migration Strategy

1. Keep existing trait-based system working
2. Implement new ComponentManager alongside
3. Add conversion utilities between old/new systems
4. Gradually migrate components to new system
5. Remove old system once migration complete

### Implementation Files

- `core/component_module.rs` - Module definitions
- `core/component_manager.rs` - Factory and instance management  
- `core/component.rs` - Updated unified Component struct
- `core/state.rs` - For `ComponentState` and related traits.
- Update `cycle_engine.rs` to use new component system
