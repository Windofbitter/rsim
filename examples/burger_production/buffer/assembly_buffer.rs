use rsim::core::component::BaseComponent;
use rsim::core::event::Event;
use rsim::core::types::ComponentValue;
use std::collections::VecDeque;

use super::super::events::{
    BufferFullEvent, BufferSpaceAvailableEvent, ItemAddedEvent, ItemDispatchedEvent,
    ItemDroppedEvent,
};

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
    fn component_id(&self) -> &String {
        &self.component_id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &["BurgerReadyEvent", "RequestItemEvent"]
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut response_events: Vec<(Box<dyn Event>, u64)> = Vec::new();

        for event in events {
            match event.event_type() {
                "BurgerReadyEvent" => {
                    if let Some(ComponentValue::String(item_id)) = event.data().get("item_id") {
                        if self.add_item(item_id.clone()) {
                            // Successfully added burger
                            let item_added = ItemAddedEvent::new(
                                self.component_id.clone(),
                                None, // Broadcast
                                "assembly".to_string(),
                                item_id.clone(),
                                self.items.len() as i32,
                            );
                            response_events.push((Box::new(item_added) as Box<dyn Event>, 0));

                            // Check if we just became full
                            if self.is_full() && !self.was_full {
                                self.was_full = true;
                                let buffer_full = BufferFullEvent::new(
                                    self.component_id.clone(),
                                    Some(vec![event.source_id().to_string()]), // Send to assembler
                                    "assembly".to_string(),
                                );
                                response_events.push((Box::new(buffer_full) as Box<dyn Event>, 0));
                            }
                        } else {
                            // Buffer is full, drop the item
                            let item_dropped = ItemDroppedEvent::new(
                                self.component_id.clone(),
                                Some(vec![event.source_id().to_string()]), // Send to assembler
                                "burger".to_string(),
                                item_id.clone(),
                                "assembly_buffer_full".to_string(),
                            );
                            response_events.push((Box::new(item_dropped) as Box<dyn Event>, 0));
                        }
                    }
                }
                "RequestItemEvent" => {
                    // Check if this request is for burger items
                    let item_type = event
                        .data()
                        .get("item_type")
                        .and_then(|v| {
                            if let ComponentValue::String(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| String::new());

                    // Only respond to burger requests
                    if item_type == "burger" {
                        if let Some(ComponentValue::String(requester_id)) =
                            event.data().get("requester_id")
                        {
                            let was_full_before = self.is_full();

                            if let Some(item_id) = self.remove_item() {
                                // Successfully removed item
                                let item_dispatched = ItemDispatchedEvent::new(
                                    self.component_id.clone(),
                                    requester_id.clone(),
                                    "burger".to_string(),
                                    item_id,
                                    true,
                                );
                                response_events
                                    .push((Box::new(item_dispatched) as Box<dyn Event>, 0));

                                // If we were full and now have space, notify assembler
                                if was_full_before && self.has_space() {
                                    self.was_full = false;
                                    let space_available = BufferSpaceAvailableEvent::new(
                                        self.component_id.clone(),
                                        None, // Broadcast to all potential producers
                                        "assembly".to_string(),
                                    );
                                    response_events
                                        .push((Box::new(space_available) as Box<dyn Event>, 0));
                                }
                            } else {
                                // No item available
                                let item_dispatched = ItemDispatchedEvent::new(
                                    self.component_id.clone(),
                                    requester_id.clone(),
                                    "burger".to_string(),
                                    String::new(),
                                    false,
                                );
                                response_events
                                    .push((Box::new(item_dispatched) as Box<dyn Event>, 0));
                            }
                        }
                    }
                }
                _ => {}
            }
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
