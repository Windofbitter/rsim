pub mod events;
pub mod components;

pub use events::*;
pub use components::*;

/// Production strategy selection
#[derive(Debug, Clone)]
pub enum ProductionStrategy {
    BufferBased,    // Current system (default)
    OrderBased,     // New demand-driven system
}

impl Default for ProductionStrategy {
    fn default() -> Self {
        ProductionStrategy::BufferBased
    }
}