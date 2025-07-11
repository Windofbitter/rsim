use crate::core::types::ComponentId;

#[derive(Debug, Clone)]
pub enum MemoryError {
    InvalidAddress(String),
    InvalidPort(String),
    MemoryNotFound(ComponentId),
    OperationFailed(String),
    TypeMismatch(String),
}