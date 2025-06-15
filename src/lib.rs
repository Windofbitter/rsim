pub mod core;
pub mod analysis;

// Re-export commonly used types
pub use crate::core::component::BaseComponent;
pub use crate::core::event::{Event, EventId, EventType};
pub use crate::core::types::ComponentId;
