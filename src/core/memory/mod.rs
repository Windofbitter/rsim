pub mod proxy;
pub mod errors;

// Re-export commonly used types
pub use proxy::TypeSafeCentralMemoryProxy;
pub use errors::MemoryError;