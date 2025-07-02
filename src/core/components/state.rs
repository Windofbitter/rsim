use std::any::Any;

/// Trait for component state that allows dynamic downcasting.
/// This enables type-safe access to component state at runtime.
pub trait ComponentState: Send {
    /// Returns a reference to the state as Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Returns a mutable reference to the state as Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Marker trait for data types that can be stored in memory components.
/// This provides compile-time type safety for memory operations.
pub trait MemoryData: Send + Clone + 'static {}

/// Errors that can occur during state management operations
#[derive(Debug, Clone)]
pub enum StateError {
    /// Attempted to downcast to incorrect type
    InvalidDowncast(String),
    /// State not found for the given component
    StateNotFound(String),
    /// Invalid state operation
    InvalidOperation(String),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::InvalidDowncast(msg) => write!(f, "Invalid downcast: {}", msg),
            StateError::StateNotFound(msg) => write!(f, "State not found: {}", msg),
            StateError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

impl std::error::Error for StateError {}

/// Helper function to safely downcast component state
pub fn downcast_state<T: ComponentState + 'static>(state: &dyn ComponentState) -> Result<&T, StateError> {
    state.as_any()
        .downcast_ref::<T>()
        .ok_or_else(|| StateError::InvalidDowncast(
            format!("Cannot downcast state to {}", std::any::type_name::<T>())
        ))
}

/// Helper function to safely downcast mutable component state
pub fn downcast_state_mut<T: ComponentState + 'static>(state: &mut dyn ComponentState) -> Result<&mut T, StateError> {
    state.as_any_mut()
        .downcast_mut::<T>()
        .ok_or_else(|| StateError::InvalidDowncast(
            format!("Cannot downcast mutable state to {}", std::any::type_name::<T>())
        ))
}

// Implement MemoryData for common types
impl MemoryData for i64 {}
impl MemoryData for f64 {}
impl MemoryData for String {}
impl MemoryData for bool {}
impl MemoryData for Vec<u8> {}