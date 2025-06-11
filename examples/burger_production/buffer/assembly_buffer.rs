use crate::core::component::{BaseComponent, ComponentState};
use crate::core::event::{ComponentValue, Event, EventType};
use std::collections::{HashMap, VecDeque};

pub struct AssemblyBuffer {
    component_id: String,
    capacity: usize,
    items: VecDeque<String>,
    was_full: bool,
}

impl AssemblyBuffer {
    pub fn new(component_id: String, capacity: usize) -> Self {
        Self {
            component_id,
            capacity,
            items: VecDeque::new(),
            was_full: false,
        }
    }

    fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }

    fn has_space(&self) -> bool {
        self.items.len() < self.capacity
    }

    fn add_item(&mut self, item_id: String) -> bool {
        if self.has_space() {
            self.items.push_back(item_id);
            true
        } else {
            false
        }
    }

    fn remove_item(&mut self) -> Option<String> {
        self.items.pop_front()
    }
}

impl BaseComponent for AssemblyBuffer {
    fn get_id(&self) -> &str {
        &self.component_id
    }

    fn get_state(&self) -> ComponentState {
        let mut state_data = HashMap::new();
        state_data.insert("capacity".to_string(), ComponentValue::Int(self.capacity as i64));
        state_data.insert("current_count".to_string(), ComponentValue::Int(self.items.len() as i64));
        state_data.insert("is_full".to_string(), ComponentValue::Bool(self.is_full()));
        
        let items_list = self.items.iter()
            .map(|id| id.clone())
            .collect::<Vec<String>>()
            .join(", ");
        state_data.insert("items".to_string(), ComponentValue::String(items_list));

        ComponentState {
            component_id: self.component_id.clone(),
            state_data,
        }
    }

    fn get_subscribed_events(&self) -> Vec<EventType> {
        vec![
            "BurgerReadyEvent".to_string(),
            "RequestItemEvent".to_string(),
        ]
    }

    fn react_atomic(&mut self, event: &Event) -> Vec<Event> {
        let mut response_events = Vec::new();
        let current_time = event.get_scheduled_time();

        match event.get_event_type() {
            "BurgerReadyEvent" => {
                if let Some(ComponentValue::String(item_id)) = event.get_data().get("item_id") {
                    if self.add_item(item_id.clone()) {
                        // Successfully added burger
                        let mut event_data = HashMap::new();
                        event_data.insert("buffer_type".to_string(), ComponentValue::String("assembly".to_string()));
                        event_data.insert("item_id".to_string(), ComponentValue::String(item_id.clone()));
                        event_data.insert("current_count".to_string(), ComponentValue::Int(self.items.len() as i64));

                        let item_added_event = Event::new(
                            "ItemAddedEvent".to_string(),
                            event_data,
                            current_time,
                            self.component_id.clone(),
                            None, // Broadcast to all subscribers
                        );
                        response_events.push(item_added_event);

                        // Check if we just became full
                        if self.is_full() && !self.was_full {
                            self.was_full = true;
                            let mut full_data = HashMap::new();
                            full_data.insert("buffer_type".to_string(), ComponentValue::String("assembly".to_string()));

                            let buffer_full_event = Event::new(
                                "BufferFullEvent".to_string(),
                                full_data,
                                current_time,
                                self.component_id.clone(),
                                Some(event.get_source_id().to_string()), // Send to assembler
                            );
                            response_events.push(buffer_full_event);
                        }
                    } else {
                        // Buffer is full, drop the item
                        let mut drop_data = HashMap::new();
                        drop_data.insert("item_type".to_string(), ComponentValue::String("burger".to_string()));
                        drop_data.insert("item_id".to_string(), ComponentValue::String(item_id.clone()));
                        drop_data.insert("reason".to_string(), ComponentValue::String("assembly_buffer_full".to_string()));

                        let item_dropped_event = Event::new(
                            "ItemDroppedEvent".to_string(),
                            drop_data,
                            current_time,
                            self.component_id.clone(),
                            Some(event.get_source_id().to_string()), // Send to assembler
                        );
                        response_events.push(item_dropped_event);
                    }
                }
            }
            "RequestItemEvent" => {
                // Only respond to requests for burgers
                if let Some(ComponentValue::String(requester_id)) = event.get_data().get("requester_id") {
                    let was_full_before = self.is_full();
                    
                    if let Some(item_id) = self.remove_item() {
                        // Successfully removed item
                        let mut dispatch_data = HashMap::new();
                        dispatch_data.insert("item_type".to_string(), ComponentValue::String("burger".to_string()));
                        dispatch_data.insert("item_id".to_string(), ComponentValue::String(item_id));
                        dispatch_data.insert("success".to_string(), ComponentValue::Bool(true));

                        let item_dispatched_event = Event::new(
                            "ItemDispatchedEvent".to_string(),
                            dispatch_data,
                            current_time,
                            self.component_id.clone(),
                            Some(requester_id.clone()),
                        );
                        response_events.push(item_dispatched_event);

                        // If we were full and now have space, notify assembler
                        if was_full_before && self.has_space() {
                            self.was_full = false;
                            let mut space_data = HashMap::new();
                            space_data.insert("buffer_type".to_string(), ComponentValue::String("assembly".to_string()));

                            let space_available_event = Event::new(
                                "BufferSpaceAvailableEvent".to_string(),
                                space_data,
                                current_time,
                                self.component_id.clone(),
                                None, // Broadcast to all potential producers
                            );
                            response_events.push(space_available_event);
                        }
                    } else {
                        // No item available
                        let mut dispatch_data = HashMap::new();
                        dispatch_data.insert("item_type".to_string(), ComponentValue::String("burger".to_string()));
                        dispatch_data.insert("item_id".to_string(), ComponentValue::String("".to_string()));
                        dispatch_data.insert("success".to_string(), ComponentValue::Bool(false));

                        let item_dispatched_event = Event::new(
                            "ItemDispatchedEvent".to_string(),
                            dispatch_data,
                            current_time,
                            self.component_id.clone(),
                            Some(requester_id.clone()),
                        );
                        response_events.push(item_dispatched_event);
                    }
                }
            }
            _ => {}
        }

        response_events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assembly_buffer_creation() {
        let buffer = AssemblyBuffer::new("assembly_buffer_1".to_string(), 5);
        assert_eq!(buffer.get_id(), "assembly_buffer_1");
        assert_eq!(buffer.capacity, 5);
        assert!(!buffer.is_full());
        assert!(buffer.has_space());
    }

    #[test]
    fn test_buffer_capacity_management() {
        let mut buffer = AssemblyBuffer::new("test_buffer".to_string(), 2);
        
        // Add items up to capacity
        assert!(buffer.add_item("burger_1".to_string()));
        assert!(buffer.add_item("burger_2".to_string()));
        assert!(buffer.is_full());
        assert!(!buffer.has_space());
        
        // Try to add when full
        assert!(!buffer.add_item("burger_3".to_string()));
        
        // Remove an item
        let removed = buffer.remove_item();
        assert_eq!(removed, Some("burger_1".to_string()));
        assert!(!buffer.is_full());
        assert!(buffer.has_space());
    }

    #[test]
    fn test_fifo_ordering() {
        let mut buffer = AssemblyBuffer::new("test_buffer".to_string(), 3);
        
        buffer.add_item("burger_1".to_string());
        buffer.add_item("burger_2".to_string());
        buffer.add_item("burger_3".to_string());
        
        assert_eq!(buffer.remove_item(), Some("burger_1".to_string()));
        assert_eq!(buffer.remove_item(), Some("burger_2".to_string()));
        assert_eq!(buffer.remove_item(), Some("burger_3".to_string()));
        assert_eq!(buffer.remove_item(), None);
    }
}