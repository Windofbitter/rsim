pub mod manager;
pub mod connection_validator;
pub mod port_validator;

// Create a unified validation module
pub use connection_validator::ConnectionValidator;
pub use port_validator::PortValidator;
pub use manager::ConnectionManager;