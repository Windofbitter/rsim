//! Component definition macros for RSim
//!
//! These macros provide high-level abstractions for defining components with less boilerplate.

/// Macro for implementing Component trait with simplified syntax
/// 
/// This macro generates both define_ports() and into_module() implementations.
/// 
/// # Example
/// ```rust
/// struct Adder;
/// 
/// impl_component!(Adder, "Adder", {
///     inputs: [a, b],
///     outputs: [sum],
///     memory: [],
///     react: |ctx, outputs| {
///         let a: i32 = ctx.inputs.get("a").unwrap_or_default();
///         let b: i32 = ctx.inputs.get("b").unwrap_or_default();
///         outputs.set("sum", a + b)?;
///         Ok(())
///     }
/// });
/// ```
#[macro_export]
macro_rules! impl_component {
    (
        $struct_name:ident, 
        $component_name:expr,
        {
            inputs: [$($input:ident),* $(,)?],
            outputs: [$($output:ident),* $(,)?],
            memory: [$($memory:ident),* $(,)?],
            react: $react_fn:expr
        }
    ) => {
        impl rsim::core::components::Component for $struct_name {
            fn define_ports() -> Vec<(String, rsim::core::components::types::PortType)> {
                port_definitions![
                    inputs: [$($input),*],
                    outputs: [$($output),*],
                    memory: [$($memory),*],
                ]
            }
            
            fn into_module() -> rsim::core::components::ProcessorModule {
                let input_ports = input_ports![$($input),*];
                let output_ports = output_ports![$($output),*];
                let memory_ports = memory_ports![$($memory),*];
                
                rsim::core::components::ProcessorModule::new(
                    $component_name,
                    input_ports,
                    output_ports,
                    memory_ports,
                    $react_fn
                )
            }
        }
    };
}

/// Macro for implementing MemoryComponent trait with simplified syntax
/// 
/// This macro generates the define_ports() implementation for memory components
/// and validates the single input/output constraint.
/// 
/// # Example
/// ```rust
/// #[derive(Clone)]
/// struct Buffer { data: i32 }
/// 
/// impl_memory_component!(Buffer, {
///     input: input,
///     output: output
/// });
/// ```
#[macro_export]
macro_rules! impl_memory_component {
    (
        $struct_name:ident,
        {
            input: $input:ident,
            output: $output:ident
        }
    ) => {
        impl rsim::core::components::MemoryComponent for $struct_name {
            fn define_ports() -> Vec<(String, rsim::core::components::types::PortType)> {
                port_definitions![
                    inputs: [$input],
                    outputs: [$output],
                ]
            }
        }
    };
}

/// Macro for creating a complete component definition
/// 
/// This macro creates both the struct and the Component implementation in one go.
/// 
/// # Example
/// ```rust
/// component! {
///     name: Adder,
///     component_name: "Adder",
///     fields: {
///         // Optional fields can go here
///     },
///     inputs: [a, b],
///     outputs: [sum],
///     memory: [],
///     react: |ctx, outputs| {
///         let a: i32 = ctx.inputs.get("a").unwrap_or_default();
///         let b: i32 = ctx.inputs.get("b").unwrap_or_default();
///         outputs.set("sum", a + b)?;
///         Ok(())
///     }
/// }
/// ```
#[macro_export]
macro_rules! component {
    (
        name: $struct_name:ident,
        component_name: $component_name:expr,
        fields: { $($field:ident: $field_type:ty),* $(,)? },
        inputs: [$($input:ident),* $(,)?],
        outputs: [$($output:ident),* $(,)?],
        memory: [$($memory:ident),* $(,)?],
        react: $react_fn:expr
    ) => {
        pub struct $struct_name {
            $(pub $field: $field_type),*
        }
        
        impl_component!($struct_name, $component_name, {
            inputs: [$($input),*],
            outputs: [$($output),*],
            memory: [$($memory),*],
            react: $react_fn
        });
    };
    
    // Simplified version without fields
    (
        name: $struct_name:ident,
        component_name: $component_name:expr,
        inputs: [$($input:ident),* $(,)?],
        outputs: [$($output:ident),* $(,)?],
        memory: [$($memory:ident),* $(,)?],
        react: $react_fn:expr
    ) => {
        pub struct $struct_name;
        
        impl_component!($struct_name, $component_name, {
            inputs: [$($input),*],
            outputs: [$($output),*],
            memory: [$($memory),*],
            react: $react_fn
        });
    };
}

/// Macro for creating a complete memory component definition
/// 
/// This macro creates both the struct and the MemoryComponent implementation.
/// 
/// # Example
/// ```rust
/// memory_component! {
///     name: Buffer,
///     fields: {
///         data: i32
///     },
///     input: input,
///     output: output,
///     cycle: |self| {
///         Some(self.data)
///     }
/// }
/// ```
#[macro_export]
macro_rules! memory_component {
    (
        name: $struct_name:ident,
        fields: { $($field:ident: $field_type:ty),* $(,)? },
        input: $input:ident,
        output: $output:ident,
        cycle: $cycle_fn:expr
    ) => {
        #[derive(Clone)]
        pub struct $struct_name {
            $(pub $field: $field_type),*
        }
        
        impl rsim::core::components::state::MemoryData for $struct_name {}
        
        impl rsim::core::components::Cycle for $struct_name {
            type Output = i32; // You may need to customize this type
            
            fn cycle(&mut self) -> Option<Self::Output> {
                ($cycle_fn)(self)
            }
        }
        
        impl_memory_component!($struct_name, {
            input: $input,
            output: $output
        });
    };
}