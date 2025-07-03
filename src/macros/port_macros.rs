//! Port definition macros for RSim components
//!
//! These macros simplify the creation of port definitions and PortSpec objects.

/// Macro for creating input port specifications
/// 
/// # Example
/// ```rust
/// let input_ports = input_ports![a, b, c];
/// // Expands to:
/// // vec![
/// //     PortSpec::input("a"),
/// //     PortSpec::input("b"), 
/// //     PortSpec::input("c"),
/// // ]
/// ```
#[macro_export]
macro_rules! input_ports {
    ($($port:ident),* $(,)?) => {
        vec![
            $(
                rsim::core::components::module::PortSpec::input(stringify!($port))
            ),*
        ]
    };
}

/// Macro for creating output port specifications
/// 
/// # Example
/// ```rust
/// let output_ports = output_ports![sum, product];
/// ```
#[macro_export]
macro_rules! output_ports {
    ($($port:ident),* $(,)?) => {
        vec![
            $(
                rsim::core::components::module::PortSpec::output(stringify!($port))
            ),*
        ]
    };
}

/// Macro for creating memory port specifications
/// 
/// # Example
/// ```rust
/// let memory_ports = memory_ports![state, buffer];
/// ```
#[macro_export]
macro_rules! memory_ports {
    ($($port:ident),* $(,)?) => {
        vec![
            $(
                rsim::core::components::module::PortSpec::memory(stringify!($port))
            ),*
        ]
    };
}

/// Macro for creating port definitions (for define_ports() method)
/// 
/// # Example
/// ```rust
/// fn define_ports() -> Vec<(String, PortType)> {
///     port_definitions![
///         inputs: [a, b],
///         outputs: [sum],
///         memory: [state]
///     ]
/// }
/// ```
#[macro_export]
macro_rules! port_definitions {
    (
        $(inputs: [$($input:ident),* $(,)?],)?
        $(outputs: [$($output:ident),* $(,)?],)?
        $(memory: [$($memory:ident),* $(,)?],)?
    ) => {
        {
            let mut ports = Vec::new();
            $(
                $(
                    ports.push((stringify!($input).to_string(), rsim::core::components::types::PortType::Input));
                )*
            )?
            $(
                $(
                    ports.push((stringify!($output).to_string(), rsim::core::components::types::PortType::Output));
                )*
            )?
            $(
                $(
                    ports.push((stringify!($memory).to_string(), rsim::core::components::types::PortType::Memory));
                )*
            )?
            ports
        }
    };
}