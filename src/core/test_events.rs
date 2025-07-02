#[cfg(test)]
mod tests {
    use super::super::values::events::*;
    use super::super::values::traits::*;
    use super::super::values::implementations::*;

    #[test]
    fn test_event_basic_functionality() {
        // Test Event creation and access
        let event = Event::new(123, 42i64);
        assert_eq!(event.timestamp, 123);
        assert_eq!(event.get_payload::<i64>().unwrap(), &42);
        assert!(event.event_id > 0);
        
        // Test unique IDs
        let event1 = Event::new(1, 42i64);
        let event2 = Event::new(1, 42i64);
        assert_ne!(event1.event_id, event2.event_id);
    }

    #[test]
    fn test_event_input_map() {
        let mut inputs = EventInputMap::new();
        inputs.insert("port1".to_string(), 100, 42i64);
        inputs.insert("port2".to_string(), 101, "hello".to_string());
        
        // Test convenience methods
        assert_eq!(inputs.get::<i64>("port1").unwrap(), 42);
        assert_eq!(inputs.get::<String>("port2").unwrap(), "hello");
        
        // Test timestamp access
        assert_eq!(inputs.get_timestamp("port1").unwrap(), 100);
        assert_eq!(inputs.get_timestamp("port2").unwrap(), 101);
        
        // Test event access
        let event = inputs.get_event("port1").unwrap();
        assert_eq!(event.timestamp, 100);
        assert_eq!(event.get_payload::<i64>().unwrap(), &42);
        
        assert!(inputs.has_input("port1"));
        assert!(!inputs.has_input("port3"));
        assert_eq!(inputs.len(), 2);
    }

    #[test]
    fn test_event_output_map() {
        let mut outputs = EventOutputMap::new_flexible(200);
        
        // Test convenience method
        outputs.set("out1", 42i64).unwrap();
        outputs.set("out2", "hello".to_string()).unwrap();
        
        let event_map = outputs.into_event_map();
        assert_eq!(event_map.len(), 2);
        
        let event1 = &event_map["out1"];
        assert_eq!(event1.timestamp, 200);
        assert_eq!(event1.get_payload::<i64>().unwrap(), &42);
        
        let event2 = &event_map["out2"];
        assert_eq!(event2.timestamp, 200);
        assert_eq!(event2.get_payload::<String>().unwrap(), "hello");
    }

    #[test]
    fn test_progressive_disclosure_api() {
        let mut inputs = EventInputMap::new();
        inputs.insert("data".to_string(), 42, 123i64);
        
        // Developer can use simple convenience method
        let simple_data: i64 = inputs.get("data").unwrap();
        assert_eq!(simple_data, 123);
        
        // Or access timestamp when needed
        let timestamp = inputs.get_timestamp("data").unwrap();
        assert_eq!(timestamp, 42);
        
        // Or get full event for advanced use cases
        let event = inputs.get_event("data").unwrap();
        assert_eq!(event.timestamp, 42);
        assert_eq!(event.get_payload::<i64>().unwrap(), &123);
        assert!(event.event_id > 0);
    }
}