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

### Architecture Overview

```rust
// Module templates (behavior definitions)
pub enum ComponentModule {
    Processing(ProcessorModule),
    Memory(MemoryModule),
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
pub struct ProcessorModule {
    pub name: String,
    pub input_ports: Vec<&'static str>,
    pub output_ports: Vec<&'static str>,
    pub memory_ports: Vec<&'static str>,
    pub evaluate_fn: fn(&HashMap<String, Event>, &mut dyn EngineMemoryProxy) -> HashMap<String, Event>,
}
```

#### MemoryModule
```rust
pub struct MemoryModule {
    pub name: String,
    pub capacity: usize,
    pub initial_data: HashMap<String, MemoryData>,
    pub read_fn: fn(&ComponentState, &str) -> Option<MemoryData>,
    pub write_fn: fn(&mut ComponentState, &str, MemoryData) -> bool,
}
```

#### Flexible Memory Data
```rust
#[derive(Debug, Clone)]
pub enum MemoryData {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Bytes(Vec<u8>),                    // Raw binary data
    Custom(Box<dyn CustomMemoryData>), // User-defined types
}

pub trait CustomMemoryData: Clone + Debug + Send + Sync {
    fn type_name(&self) -> &'static str;
    fn as_any(&self) -> &dyn std::any::Any;
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
    input_ports: vec!["a", "b"],
    output_ports: vec!["sum"],
    evaluate_fn: |inputs, _memory| {
        let a = inputs.get("a").and_then(|v| v.as_int()).unwrap_or(0);
        let b = inputs.get("b").and_then(|v| v.as_int()).unwrap_or(0);
        HashMap::from([("sum".to_string(), ComponentValue::Int(a + b))])
    },
};
component_manager.register_module(ComponentModule::Processing(adder_module))?;

// 2. Register memory module with flexible data
let ram_module = MemoryModule {
    name: "ram".to_string(),
    capacity: 1024,
    initial_data: HashMap::from([
        ("0x0000".to_string(), MemoryData::Int(42)),           // Boot vector
        ("0x0004".to_string(), MemoryData::Float(3.14)),       // Pi constant
        ("config".to_string(), MemoryData::Bytes(vec![1,2,3])), // Binary data
        ("status".to_string(), MemoryData::Bool(true)),        // Status flag
    ]),
    read_fn: |state, addr| { /* read implementation */ },
    write_fn: |state, addr, data| { /* write implementation */ },
};
component_manager.register_module(ComponentModule::Memory(ram_module))?;

// 3. Create multiple instances
let adder1 = component_manager.create_component("adder", Some("adder_cpu1".to_string()))?;
let adder2 = component_manager.create_component("adder", Some("adder_cpu2".to_string()))?;
let ram1 = component_manager.create_component("ram", Some("main_memory".to_string()))?;
let ram2 = component_manager.create_component_auto_id("ram")?; // "ram_0"
```

### Benefits

1. **Performance**: Direct struct access, no trait object overhead
2. **Multiple Instances**: Easy creation of many instances from same template
3. **Unified API**: Single registration and creation path
4. **Better Debugging**: Concrete types, no type erasure
5. **Extensibility**: Easy to add new component types through modules
6. **Memory Efficiency**: Shared behavior definitions, unique instance state
7. **Flexible Memory Data**: Support for any data type through extensible enum

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
- Update `cycle_engine.rs` to use new component system