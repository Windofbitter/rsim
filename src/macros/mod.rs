//! RSim Component Macros
//!
//! This module provides declarative macros to reduce boilerplate in component definitions.
//! These macros are simpler than procedural macros but still provide significant value.

pub mod component_macros;
pub mod memory_macros;
pub mod port_macros;

// Re-export all macros
pub use component_macros::*;
pub use memory_macros::*;
pub use port_macros::*;