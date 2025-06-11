use rsim::core::component::BaseComponent;
use rsim::core::event::Event;
use rsim::core::types::ComponentId;
use std::collections::VecDeque;

use crate::events::{
    BreadReadyEvent, BufferFullEvent, BufferSpaceAvailableEvent, ItemAddedEvent,
    ItemDispatchedEvent, ItemDroppedEvent, RequestItemEvent,
};

#[derive(Debug)]
pub struct CookedBreadBuffer {
    component_id: ComponentId,
    capacity: usize,
    items: VecDeque<String>, // item_id queue for FIFO
    was_full: bool,
}

impl CookedBreadBuffer {
    pub fn new(component_id: ComponentId, capacity: Option<usize>) -> Self {
        Self {
            component_id,
            capacity: capacity.unwrap_or(5),
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

impl BaseComponent for CookedBreadBuffer {
    fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &["BreadReadyEvent", "RequestItemEvent"]
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        for event in events {
            match event.event_type() {
                "BreadReadyEvent" => {
                    // Try to extract item_id from event data
                    if let Some(data) = event.data().get("item_id") {
                        if let rsim::core::types::ComponentValue::String(item_id) = data {
                            if self.add_item(item_id.clone()) {
                                // Item successfully added
                                let item_added_event = ItemAddedEvent::new(
                                    self.component_id.clone(),
                                    None, // Broadcast
                                    "cooked_bread".to_string(),
                                    item_id.clone(),
                                    self.items.len() as i32,
                                );
                                output_events.push((Box::new(item_added_event), 0));

                                // Check if buffer became full
                                if self.is_full() && !self.was_full {
                                    self.was_full = true;
                                    let buffer_full_event = BufferFullEvent::new(
                                        self.component_id.clone(),
                                        Some(vec![event.source_id().clone()]),
                                        "cooked_bread".to_string(),
                                    );
                                    output_events.push((Box::new(buffer_full_event), 0));
                                }
                            } else {
                                // Buffer is full, drop the item
                                let item_dropped_event = ItemDroppedEvent::new(
                                    self.component_id.clone(),
                                    Some(vec![event.source_id().clone()]),
                                    "cooked_bread".to_string(),
                                    item_id.clone(),
                                    "buffer_full".to_string(),
                                );
                                output_events.push((Box::new(item_dropped_event), 0));
                            }
                        }
                    }
                }
                "RequestItemEvent" => {
                    if let Some(requester_id_value) = event.data().get("requester_id") {
                        if let rsim::core::types::ComponentValue::String(requester_id) =
                            requester_id_value
                        {
                            if let Some(item_id) = self.remove_item() {
                                // Create a response event with the item
                                let item_dispatched_event = ItemDispatchedEvent::new(
                                    self.component_id.clone(),
                                    requester_id.clone(),
                                    "cooked_bread".to_string(),
                                    item_id,
                                    true,
                                );
                                output_events.push((Box::new(item_dispatched_event), 0));

                                // Check if buffer was full and now has space
                                if self.was_full && self.has_space() {
                                    self.was_full = false;
                                    let space_available_event = BufferSpaceAvailableEvent::new(
                                        self.component_id.clone(),
                                        None, // Broadcast to all upstream producers
                                        "cooked_bread".to_string(),
                                    );
                                    output_events.push((Box::new(space_available_event), 0));
                                }
                            } else {
                                let item_dispatched_event = ItemDispatchedEvent::new(
                                    self.component_id.clone(),
                                    requester_id.clone(),
                                    "cooked_bread".to_string(),
                                    "".to_string(),
                                    false,
                                );
                                output_events.push((Box::new(item_dispatched_event), 0));
                            }
                        }
                    }
                }
                _ => {} // Ignore unknown events
            }
        }

        output_events
    }
}