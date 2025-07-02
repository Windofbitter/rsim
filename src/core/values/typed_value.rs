use std::any::{Any, TypeId};

/// Type-erased but type-safe container for component values
#[derive(Debug)]
pub struct TypedValue {
    data: Box<dyn Any + Send + Sync>,
    clone_fn: fn(&dyn Any) -> Box<dyn Any + Send + Sync>,
    type_name: &'static str,
    type_id: TypeId,
}

impl TypedValue {
    /// Create a new typed value
    pub fn new<T: Send + Sync + Clone + 'static>(value: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            data: Box::new(value),
            clone_fn: |any| {
                let typed = any.downcast_ref::<T>().expect("Type mismatch in clone_fn");
                Box::new(typed.clone())
            },
        }
    }
    
    /// Get a reference to the contained value
    pub fn get<T: 'static>(&self) -> Result<&T, String> {
        if TypeId::of::<T>() != self.type_id {
            return Err(format!(
                "Type mismatch: expected {}, found {}", 
                std::any::type_name::<T>(), 
                self.type_name
            ));
        }
        
        self.data.downcast_ref::<T>()
            .ok_or_else(|| format!("Failed to downcast to {}", std::any::type_name::<T>()))
    }
    
    /// Consume the typed value and return the contained value
    pub fn into_inner<T: 'static>(self) -> Result<T, String> {
        if TypeId::of::<T>() != self.type_id {
            return Err(format!(
                "Type mismatch: expected {}, found {}", 
                std::any::type_name::<T>(), 
                self.type_name
            ));
        }
        
        self.data.downcast::<T>()
            .map(|boxed| *boxed)
            .map_err(|_| format!("Failed to downcast to {}", std::any::type_name::<T>()))
    }
    
    /// Get the type name of the contained value
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }
    
    /// Get the type ID of the contained value
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
    
    /// Check if the contained value is of type T
    pub fn is_type<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }
}

impl Clone for TypedValue {
    fn clone(&self) -> Self {
        Self {
            data: (self.clone_fn)(self.data.as_ref()),
            clone_fn: self.clone_fn,
            type_name: self.type_name,
            type_id: self.type_id,
        }
    }
}

/// Helper trait for types that can be used in the typed system
pub trait TypedData: Send + Sync + Clone + 'static {}

// Implement for common types
impl TypedData for i64 {}
impl TypedData for f64 {}
impl TypedData for String {}
impl TypedData for bool {}
impl TypedData for u64 {}
impl TypedData for i32 {}
impl TypedData for f32 {}
impl TypedData for u32 {}
impl<T: TypedData> TypedData for Vec<T> {}
impl<T: TypedData> TypedData for Option<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_value_basic() {
        let value = TypedValue::new(42i64);
        assert_eq!(value.get::<i64>().unwrap(), &42);
        assert!(value.is_type::<i64>());
        assert!(!value.is_type::<String>());
    }
    
    #[test]
    fn test_typed_value_type_mismatch() {
        let value = TypedValue::new(42i64);
        let result = value.get::<String>();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type mismatch"));
    }
}