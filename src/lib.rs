pub mod core;
pub mod macros;

#[cfg(test)]
mod macro_tests;

#[cfg(test)]
mod test_phase3;

#[cfg(test)]
mod test_phase4;

// Re-export commonly used types
pub use crate::core::types::ComponentId;
pub use crate::core::{Inputs, Outputs, React, Component, MemoryComponent, Cycle};

// Re-export macros
pub use crate::macros::*;
