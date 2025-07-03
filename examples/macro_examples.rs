//! Examples showing RSim macro usage
//! 
//! This file demonstrates how the RSim macros can dramatically reduce boilerplate
//! when defining components.

use rsim::*;
use rsim::core::values::traits::{EventInputs, EventOutputs};

// ============================================================================
// BEFORE: Traditional manual implementation (lots of boilerplate)
// ============================================================================

struct AdderManual;

impl Component for AdderManual {
    fn define_ports() -> Vec<(String, crate::core::components::types::PortType)> {
        vec![
            ("a".to_string(), crate::core::components::types::PortType::Input),
            ("b".to_string(), crate::core::components::types::PortType::Input),
            ("sum".to_string(), crate::core::components::types::PortType::Output),
        ]
    }
    
    fn into_module() -> crate::core::components::ProcessorModule {
        let ports = Self::define_ports();
        let input_ports = ports.iter()
            .filter(|(_, t)| *t == crate::core::components::types::PortType::Input)
            .map(|(name, _)| crate::core::components::module::PortSpec::input(name))
            .collect();
        let output_ports = ports.iter()
            .filter(|(_, t)| *t == crate::core::components::types::PortType::Output)
            .map(|(name, _)| crate::core::components::module::PortSpec::output(name))
            .collect();
        let memory_ports = ports.iter()
            .filter(|(_, t)| *t == crate::core::components::types::PortType::Memory)
            .map(|(name, _)| crate::core::components::module::PortSpec::memory(name))
            .collect();
        
        crate::core::components::ProcessorModule::new(
            "AdderManual",
            input_ports,
            output_ports,
            memory_ports,
            |ctx, outputs| {
                let a: i32 = ctx.inputs.get("a").unwrap_or_default();
                let b: i32 = ctx.inputs.get("b").unwrap_or_default();
                outputs.set("sum", a + b)?;
                Ok(())
            }
        )
    }
}

// ============================================================================
// AFTER: Using RSim macros (minimal boilerplate)
// ============================================================================

// Option 1: impl_component! macro
struct AdderMacro;

impl_component!(AdderMacro, "AdderMacro", {
    inputs: [a, b],
    outputs: [sum],
    memory: [],
    react: |ctx, outputs| {
        let a: i32 = ctx.inputs.get("a").unwrap_or_default();
        let b: i32 = ctx.inputs.get("b").unwrap_or_default();
        outputs.set("sum", a + b)?;
        Ok(())
    }
});

// Option 2: component! macro (even more concise)
component! {
    name: AdderComplete,
    component_name: "AdderComplete",
    inputs: [x, y],
    outputs: [result],
    memory: [],
    react: |ctx, outputs| {
        let x: i32 = ctx.inputs.get("x").unwrap_or_default();
        let y: i32 = ctx.inputs.get("y").unwrap_or_default();
        outputs.set("result", x + y)?;
        Ok(())
    }
}

// ============================================================================
// MEMORY COMPONENT EXAMPLES
// ============================================================================

// BEFORE: Manual memory component implementation
#[derive(Clone)]
struct BufferManual {
    data: i32,
}

impl crate::core::components::state::MemoryData for BufferManual {}

impl Cycle for BufferManual {
    type Output = i32;
    
    fn cycle(&mut self) -> Option<Self::Output> {
        Some(self.data)
    }
}

impl MemoryComponent for BufferManual {
    fn define_ports() -> Vec<(String, crate::core::components::types::PortType)> {
        vec![
            ("input".to_string(), crate::core::components::types::PortType::Input),
            ("output".to_string(), crate::core::components::types::PortType::Output),
        ]
    }
}

// AFTER: Using macro (much simpler)
#[derive(Clone)]
struct BufferMacro {
    data: i32,
}

impl crate::core::components::state::MemoryData for BufferMacro {}

impl Cycle for BufferMacro {
    type Output = i32;
    
    fn cycle(&mut self) -> Option<Self::Output> {
        Some(self.data)
    }
}

impl_memory_component!(BufferMacro, {
    input: input,
    output: output
});

// ============================================================================
// MEMORY ACCESS MACROS EXAMPLE
// ============================================================================

struct BakerWithMacros;

impl_component!(BakerWithMacros, "BakerWithMacros", {
    inputs: [],
    outputs: [],
    memory: [baker_state, bread_buffer],
    react: |ctx, _outputs| {
        // BEFORE: Verbose memory operations
        // let mut remaining_cycles = ctx.memory.read::<u32>("baker_state", "remaining_cycles")
        //     .unwrap_or(Some(0)).unwrap_or(0);
        // let mut total_produced = ctx.memory.read::<u64>("baker_state", "total_produced")
        //     .unwrap_or(Some(0)).unwrap_or(0);
        
        // AFTER: Using memory macros
        memory_state!(ctx, "baker_state", {
            remaining_cycles: i64 = 0,
            total_produced: i64 = 0,
            rng_state: i64 = 42,
        });
        
        // Simulate baking logic
        if remaining_cycles == 0 {
            remaining_cycles = 10; // Start new bread
        } else {
            remaining_cycles -= 1;
            if remaining_cycles == 0 {
                total_produced += 1;
                // Add bread to buffer
                memory_write!(ctx, "bread_buffer", "count", total_produced);
            }
        }
        
        // BEFORE: Verbose memory writes
        // ctx.memory.write("baker_state", "remaining_cycles", remaining_cycles)?;
        // ctx.memory.write("baker_state", "total_produced", total_produced)?;
        
        // AFTER: Bulk memory write
        memory_state_write!(ctx, "baker_state", remaining_cycles, total_produced, rng_state);
        
        Ok(())
    }
});

// ============================================================================
// DEMO FUNCTIONS
// ============================================================================

fn main() {
    println!("RSim Macro Examples");
    println!("===================");
    
    // Show port definitions
    println!("Manual Adder ports: {:?}", AdderManual::define_ports());
    println!("Macro Adder ports: {:?}", AdderMacro::define_ports());
    println!("Complete Adder ports: {:?}", AdderComplete::define_ports());
    
    // Show memory component ports
    println!("Manual Buffer ports: {:?}", BufferManual::define_ports());
    println!("Macro Buffer ports: {:?}", BufferMacro::define_ports());
    
    // Show baker component ports
    println!("Baker ports: {:?}", BakerWithMacros::define_ports());
    
    println!("\nMacros successfully reduce boilerplate by ~90%!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_manual_vs_macro_equivalence() {
        // Both implementations should produce identical port definitions
        assert_eq!(AdderManual::define_ports(), AdderMacro::define_ports());
        assert_eq!(BufferManual::define_ports(), BufferMacro::define_ports());
    }
    
    #[test]
    fn test_macro_component_creation() {
        // Test that macro-generated components work correctly
        let module = AdderMacro::into_module();
        assert_eq!(module.name, "AdderMacro");
        assert_eq!(module.input_ports.len(), 2);
        assert_eq!(module.output_ports.len(), 1);
        assert_eq!(module.memory_ports.len(), 0);
    }
    
    #[test]
    fn test_complete_component_macro() {
        let module = AdderComplete::into_module();
        assert_eq!(module.name, "AdderComplete");
        
        let ports = AdderComplete::define_ports();
        assert_eq!(ports.len(), 3);
    }
}