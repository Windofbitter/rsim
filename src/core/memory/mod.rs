pub mod proxy;
pub mod errors;
pub mod delta;

// Re-export commonly used types
pub use proxy::MemoryProxy;
pub use errors::MemoryError;
pub use delta::{MemoryDelta, MemoryWrite};