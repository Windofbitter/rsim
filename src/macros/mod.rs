//! RSim Component Macros
//!
//! This module provides declarative macros to reduce boilerplate in component definitions.
//! These macros are simpler than procedural macros but still provide significant value.

pub mod component_macros;
pub mod memory_macros;
pub mod port_macros;

// Re-export all macros 
// Note: These are exported via #[macro_export] in each module
// This re-export allows internal use within the rsim crate