//! Memory access macros for RSim components
//!
//! These macros simplify memory read/write operations in component implementations.

/// Macro for reading from memory with default values
/// 
/// This macro simplifies the verbose memory read pattern:
/// ```rust
/// // Instead of:
/// let value = ctx.memory.read::<Type>("port", "address").unwrap_or(Some(default)).unwrap_or(default);
/// 
/// // Use:
/// memory_read!(ctx, "port", "address", value: Type = default);
/// ```
#[macro_export]
macro_rules! memory_read {
    ($ctx:expr, $port:expr, $address:expr, $var:ident: $type:ty = $default:expr) => {
        let mut $var: $type = $ctx.memory.read::<$type>($port, $address)
            .unwrap_or(Some($default))
            .unwrap_or($default);
    };
}

/// Macro for reading from memory with error propagation
/// 
/// This variant propagates errors instead of using default values:
/// ```rust
/// // Use for critical reads where defaults aren't appropriate:
/// memory_read_or_error!(ctx, "port", "address", value: Type)?;
/// ```
#[macro_export]
macro_rules! memory_read_or_error {
    ($ctx:expr, $port:expr, $address:expr, $var:ident: $type:ty) => {
        let mut $var: $type = $ctx.memory.read::<$type>($port, $address)
            .map_err(|e| format!("Failed to read memory port '{}' address '{}': {}", $port, $address, e))?
            .ok_or_else(|| format!("No value found at memory port '{}' address '{}'", $port, $address))?;
    };
}

/// Macro for writing to memory
/// 
/// This macro simplifies memory write operations:
/// ```rust
/// // Instead of:
/// ctx.memory.write("port", "address", value)?;
/// 
/// // Use:
/// memory_write!(ctx, "port", "address", value);
/// ```
#[macro_export]
macro_rules! memory_write {
    ($ctx:expr, $port:expr, $address:expr, $value:expr) => {
        $ctx.memory.write($port, $address, $value)?;
    };
}

/// Macro for reading multiple memory fields with a single port
/// 
/// This macro reduces boilerplate for reading multiple fields from the same memory port:
/// ```rust
/// // Instead of multiple memory_read! calls:
/// memory_read!(ctx, "state", "field1", field1: u32 = 0);
/// memory_read!(ctx, "state", "field2", field2: u64 = 0);
/// 
/// // Use:
/// memory_state!(ctx, "state", {
///     field1: u32 = 0,
///     field2: u64 = 0,
/// });
/// ```
#[macro_export]
macro_rules! memory_state {
    ($ctx:expr, $port:expr, { $($field:ident: $type:ty = $default:expr),* $(,)? }) => {
        $(
            memory_read!($ctx, $port, stringify!($field), $field: $type = $default);
        )*
    };
}

/// Macro for writing multiple memory fields with a single port
/// 
/// This macro reduces boilerplate for writing multiple fields to the same memory port:
/// ```rust
/// // Instead of multiple memory_write! calls:
/// memory_write!(ctx, "state", "field1", field1);
/// memory_write!(ctx, "state", "field2", field2);
/// 
/// // Use:
/// memory_state_write!(ctx, "state", field1, field2);
/// ```
#[macro_export]
macro_rules! memory_state_write {
    ($ctx:expr, $port:expr, $($field:ident),* $(,)?) => {
        $(
            memory_write!($ctx, $port, stringify!($field), $field);
        )*
    };
}

/// Macro for complete memory state management (read + write)
/// 
/// This macro combines reading and writing for complete state management:
/// ```rust
/// memory_state_rw!(ctx, "baker_state", {
///     remaining_cycles: u32 = 0,
///     total_produced: u64 = 0,
///     rng_state: u64 = 42,
/// });
/// ```
#[macro_export]
macro_rules! memory_state_rw {
    ($ctx:expr, $port:expr, { $($field:ident: $type:ty = $default:expr),* $(,)? }) => {
        // Read all fields
        $(
            memory_read!($ctx, $port, stringify!($field), $field: $type = $default);
        )*
        
        // Create a closure that writes all fields back to memory
        let write_state = || -> Result<(), String> {
            $(
                memory_write!($ctx, $port, stringify!($field), $field);
            )*
            Ok(())
        };
        
        // You can call write_state() at the end of your logic
        let _write_state = write_state;
    };
}