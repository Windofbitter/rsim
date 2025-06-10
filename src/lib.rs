pub mod core;
pub mod components;
pub mod utils;
pub mod builders;

// Re-export commonly used types
pub use crate::core::component::BaseComponent;
pub use crate::core::types::ComponentId;
pub use crate::core::event::{Event, EventType, EventId};