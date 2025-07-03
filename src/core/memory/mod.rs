pub mod proxy;
pub mod errors;

// Re-export commonly used types
pub use proxy::MemoryProxy;
pub use errors::MemoryError;