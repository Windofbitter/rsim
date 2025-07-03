pub mod core;
pub mod macros;

#[cfg(test)]
mod macro_tests;

// Re-export commonly used types
pub use crate::core::types::ComponentId;
pub use crate::core::{Inputs, Outputs, React, Component, MemoryComponent, Cycle};

// Re-export macros
pub use crate::macros::*;
