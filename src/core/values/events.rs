use std::sync::atomic::{AtomicU64, Ordering};
use crate::core::values::typed_value::TypedValue;

/// Global event ID counter for unique event identification
static EVENT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a unique event ID
fn next_event_id() -> u64 {
    EVENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Event wrapper containing timestamp, unique ID, and typed payload
#[derive(Debug, Clone)]
pub struct Event {
    pub event_id: u64,
    pub timestamp: u64,
    pub payload: TypedValue,
}

impl Event {
    /// Create a new event with auto-generated timestamp and ID
    pub fn new<T: Send + Sync + Clone + 'static>(timestamp: u64, payload: T) -> Self {
        Self {
            event_id: next_event_id(),
            timestamp,
            payload: TypedValue::new(payload),
        }
    }
    
    /// Create event from existing TypedValue
    pub fn from_typed_value(timestamp: u64, payload: TypedValue) -> Self {
        Self {
            event_id: next_event_id(),
            timestamp,
            payload,
        }
    }
    
    /// Get the payload as a specific type
    pub fn get_payload<T: 'static>(&self) -> Result<&T, String> {
        self.payload.get::<T>()
    }
    
    /// Extract the payload as a specific type
    pub fn into_payload<T: 'static>(self) -> Result<T, String> {
        self.payload.into_inner::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::new(123, 42i64);
        assert_eq!(event.timestamp, 123);
        assert_eq!(event.get_payload::<i64>().unwrap(), &42);
        assert!(event.event_id > 0);
    }
    
    #[test]
    fn test_event_unique_ids() {
        let event1 = Event::new(1, 42i64);
        let event2 = Event::new(1, 42i64);
        assert_ne!(event1.event_id, event2.event_id);
    }
}