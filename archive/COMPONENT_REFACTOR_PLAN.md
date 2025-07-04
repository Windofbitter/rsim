# Component Model Refactor Plan

## Requirements

### Processor Components
- **Multiple input ports**: Any number of typed inputs
- **Multiple output ports**: Any number of outputs, but all share same output type
- **Memory ports**: Optional memory access via parametric trait
- **Manual port definition**: Components must explicitly define their ports
- **Full manual approach**: No derive macros or meta-programming

### Memory Components  
- **Single input/output**: One typed input and output field
- **External access**: Other components can read/write via memory connections
- **Cycle trait**: Updates internal state per cycle

### Example Target API
```rust
// Processor with inputs, outputs, and memory
struct CPU {
    pub instruction: Instruction,
    pub data: Data,
    pub result: Result,
    pub status: Result,
    pub cache: String,     // Memory port name
}

impl Component for CPU {
    fn define_ports() -> Vec<(String, PortType)> {
        vec![
            ("instruction".to_string(), PortType::Input),
            ("data".to_string(), PortType::Input),
            ("result".to_string(), PortType::Output),
            ("status".to_string(), PortType::Output),
            ("cache".to_string(), PortType::Memory),
        ]
    }
    
    fn into_module() -> ProcessorModule {
        ProcessorModule::new("CPU", Self::define_ports(), |ctx, outputs| {
            // Extract inputs from simulation context
            let instruction: Instruction = ctx.inputs.get("instruction")?;
            let data: Data = ctx.inputs.get("data")?;
            
            // Create component instance
            let mut cpu = CPU { 
                instruction, data,
                result: Default::default(),
                status: Default::default(),
                cache: "cache".to_string(),
            };
            
            // Call React implementation
            if let Some(output) = cpu.react(&mut ctx.memory) {
                outputs.set("result", output.clone())?;
                outputs.set("status", output.clone())?;
            }
            
            Ok(())
        })
    }
}

impl React<&mut MemoryProxy> for CPU {
    type Output = Result;
    fn react(&mut self, memory: &mut MemoryProxy) -> Option<Self::Output> {
        // Access inputs as struct fields
        let inst = &self.instruction;
        
        // Access memory through proxy
        let cached: Option<u32> = memory.read(&self.cache, "0x1000")?;
        memory.write(&self.cache, "0x2000", 42u32)?;
        
        Some(result)
    }
}

// Simple processor without memory
struct Adder {
    pub a: i32,
    pub b: i32,
    pub sum: i32,
}

impl Component for Adder {
    fn define_ports() -> Vec<(String, PortType)> {
        vec![
            ("a".to_string(), PortType::Input),
            ("b".to_string(), PortType::Input),
            ("sum".to_string(), PortType::Output),
        ]
    }
    
    fn into_module() -> ProcessorModule {
        ProcessorModule::new("Adder", Self::define_ports(), |ctx, outputs| {
            // Extract inputs from simulation context
            let a: i32 = ctx.inputs.get("a")?;
            let b: i32 = ctx.inputs.get("b")?;
            
            // Create component instance
            let mut adder = Adder { a, b, sum: 0 };
            
            // Call React implementation
            if let Some(output) = adder.react() {
                outputs.set("sum", output)?;
            }
            
            Ok(())
        })
    }
}

impl React for Adder {
    type Output = i32;
    fn react(&mut self) -> Option<Self::Output> {
        Some(self.a + self.b)
    }
}

// Memory component
struct Buffer {
    pub data_in: Data,
    pub data_out: Data,
    storage: Vec<Data>,
}

impl MemoryComponent for Buffer {
    fn define_ports() -> Vec<(String, PortType)> {
        vec![
            ("data_in".to_string(), PortType::Input),
            ("data_out".to_string(), PortType::Output),
        ]
    }
    
    fn into_memory_module() -> MemoryModule<Self> {
        MemoryModule::new("Buffer", Self::define_ports())
    }
}

impl Cycle for Buffer {
    fn cycle(&mut self) -> Option<Data> {
        self.storage.last().cloned()
    }
}
```

## Proposed Changes

### 1. New Traits
```rust
// Parametric processor reaction trait
trait React<Ctx = ()> {
    type Output;
    fn react(&mut self, ctx: Ctx) -> Option<Self::Output>;
}

// Memory update trait  
trait Cycle {
    type Output;
    fn cycle(&mut self) -> Option<Self::Output>;
}

// Manual port definition trait
trait Component {
    fn define_ports() -> Vec<(String, PortType)>;
    fn into_module() -> ProcessorModule;
}

trait MemoryComponent: Cycle {
    fn define_ports() -> Vec<(String, PortType)>;
    fn into_memory_module() -> MemoryModule<Self>;
}
```

### 2. Manual Component Implementation

#### For Processor Components:
- Manually implement `Component` trait with `define_ports()` and `into_module()` methods
- Port names are explicitly defined as strings in `define_ports()`
- Components implement `React` trait for their business logic
- The `into_module()` method creates the evaluation function that bridges the simulation engine to your component
- **NO derive macros or meta-programming** - all implementations are explicit

**Three-Step Implementation Process:**
1. **Define struct** with public fields for ports
2. **Implement React trait** for your component logic
3. **Implement Component trait** with manual port definitions and module bridge

**Full Manual Architecture:**
```rust
// Step 1: Define component struct
struct CPU {
    pub instruction: Instruction,
    pub data: Data,
    pub result: Result,
    pub status: Result,
    pub cache: String,
}

// Step 2: Implement React trait (your business logic)
impl React<&mut MemoryProxy> for CPU {
    type Output = Result;
    fn react(&mut self, memory: &mut MemoryProxy) -> Option<Self::Output> {
        let result = process_instruction(&self.instruction, &self.data);
        memory.write(&self.cache, "addr", result.clone())?;
        Some(result)
    }
}

// Step 3: Implement Component trait (simulation integration)
impl Component for CPU {
    fn define_ports() -> Vec<(String, PortType)> {
        vec![
            ("instruction".to_string(), PortType::Input),
            ("data".to_string(), PortType::Input),
            ("result".to_string(), PortType::Output),
            ("status".to_string(), PortType::Output),
            ("cache".to_string(), PortType::Memory),
        ]
    }

    fn into_module() -> ProcessorModule {
        ProcessorModule::new("CPU", Self::define_ports(), |ctx, outputs| {
            // Manual bridge: Extract inputs from simulation context
            let instruction: Instruction = ctx.inputs.get("instruction")?;
            let data: Data = ctx.inputs.get("data")?;
            
            // Manual instantiation
            let mut cpu = CPU { 
                instruction, data,
                result: Default::default(),
                status: Default::default(),
                cache: "cache".to_string(),
            };
            
            // Call React implementation and handle outputs
            if let Some(output) = cpu.react(&mut ctx.memory) {
                outputs.set("result", output.clone())?;
                outputs.set("status", output.clone())?;
            }
            
            Ok(())
        })
    }
}
```

#### For Memory Components:
- Manually implement `MemoryComponent` trait with `define_ports()` and `into_memory_module()`
- Expect exactly one input and one output port
- Handle memory read/write connections from other components
- Implement `Cycle` trait for state updates

## Implementation Strategy

### Phase 1: Core Traits
1. Add `React` and `Cycle` traits to `core/components/traits.rs`
2. Define `Component` and `MemoryComponent` traits with manual port definition
3. Create conversion helpers to existing module types

### Phase 2: Manual Implementation Helpers
1. Create helper functions for common evaluation patterns
2. Add validation for port definitions
3. Implement `into_module()` methods for trait conversions

### Phase 3: Integration
1. Update examples to use new manual API
2. Add component implementation tests
3. Document manual implementation patterns

## Key Design Decisions

- **Parametric React Trait**: `React<Ctx = ()>` enables progressive complexity
- **Manual Port Definition**: Explicit port specification without meta-programming
- **Single Output Type**: All output ports share same type for consistency
- **Backward Compatible**: Built on top of existing `ProcessorModule`/`MemoryModule`

## Context System

```rust
// Simple components (80% of cases)
impl React for Adder {
    fn react(&mut self) -> Option<i32> { Some(self.a + self.b) }
}

// Advanced components (20% of cases) 
impl React<&mut SimulationContext> for CPU {
    fn react(&mut self, ctx: &mut SimulationContext) -> Option<Result> {
        let data = ctx.get::<Data>("input")?;           // Simple access
        let timing = ctx.get_timestamp("input")?;       // Timing access
        let cached = ctx.memory().read(&self.cache)?;   // Memory access
        Some(process(data, timing, cached))
    }
}
```

**Manual Implementation**:
- `React` → calls `component.react()`  
- `React<&mut SimulationContext>` → creates context, calls `component.react(&mut ctx)`

## Benefits

- **No Meta-programming**: Completely avoids derive macros and meta-programming
- **Type Safety**: Compile-time validation of connections
- **Clear Intent**: Explicit port definitions show component interface
- **Full Control**: Complete control over component behavior and module bridge
- **Simple Implementation**: Straightforward three-step component creation process
- **Explicit Bridges**: Manual `into_module()` implementation makes simulation integration clear
- **No Hidden Magic**: All component behavior is explicitly written and visible

## Current Complexity Analysis & Cuts

### What We Can Remove (~30-40% reduction):

1. **Dual Context System** - Unify `EvaluationContext` and `LegacyEvaluationContext`
2. **Multiple Input/Output Trait Hierarchies** - Merge `TypedInputs/EventInputs` into single interface
3. **Manual Port Specification** - Eliminate `PortSpec` builders, `required` attribute, and `description` field (use `Option<T>` for optionality, doc comments for documentation)
4. **Component Manager Complexity** - Simplify auto-ID generation and validation logic
5. **Complex Module Wrapping** - Simplify `ComponentModule` enum through trait-based approach

### Current Pain Points to Address:

- **src/core/values/traits.rs:6-82** - Complex trait hierarchies for inputs/outputs
- **src/core/components/module.rs:39-117** - Manual port specification boilerplate
- **src/core/components/manager.rs** - Complex instance creation logic
- **src/core/components/module.rs:268-317** - Heavy module wrapper enum

### What to Keep:

- Core execution engine (`src/core/execution/`)
- Memory proxy system (simplified)
- Connection management (streamlined validation)
- Component registry (reduced responsibilities)
- Type system (`src/core/types.rs`) - minimal changes

## Simplified Port Specification:

**PortSpec reduced to name and port type only**:
```rust
pub struct PortSpec {
    pub name: String,
    pub port_type: PortType,
}
```

## Refactor Impact:

**Before**: Manual port specs, complex traits, dual contexts, heavy component management
**After**: Explicit port definitions, simplified traits, unified interfaces, streamlined runtime

## Full Manual Implementation Summary:

The refactored approach eliminates ALL meta-programming and provides a clear, explicit component definition process:

### Component Definition Steps:
1. **Define struct** with public fields for ports
2. **Implement React trait** for component business logic  
3. **Implement Component trait** with manual port definitions and module bridge

### Key Manual Elements:
- **Port Definitions**: Explicitly list all ports in `define_ports()`
- **Module Bridge**: Manually implement `into_module()` to connect simulation engine
- **Input Extraction**: Manually extract inputs from simulation context
- **Output Handling**: Manually set outputs in simulation context
- **Instance Creation**: Manually create component instances with proper field values

### No Automation:
- No derive macros
- No field name reflection
- No automatic port detection
- No generated code

This approach provides **complete transparency** and **full control** over component behavior while maintaining **type safety** and **clear interfaces**.