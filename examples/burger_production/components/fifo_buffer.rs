use rsim::core::component::BaseComponent;
use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue, SimulationTime};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

use crate::events::{
    ItemAddedEvent, BufferFullEvent, BufferSpaceAvailableEvent, RequestItemEvent,
    ITEM_ADDED_EVENT, BUFFER_FULL_EVENT, BUFFER_SPACE_AVAILABLE_EVENT, REQUEST_ITEM_EVENT,
    MEAT_READY_EVENT, BREAD_READY_EVENT, BURGER_READY_EVENT, PLACE_ORDER_EVENT
};

#[derive(Debug, Clone)]
pub struct BufferItem {
    pub item_type: String,
    pub item_id: String,
    pub timestamp: SimulationTime,
}

/// Generic FIFO buffer with common functionality
pub struct GenericFifoBuffer {
    pub component_id: ComponentId,
    pub capacity: u32,
    pub items: VecDeque<BufferItem>,
    pub subscribers: Vec<ComponentId>,
    pub is_full: bool,
    pub was_empty: bool,
    pub expected_item_type: String,
}

impl GenericFifoBuffer {
    pub fn new(component_id: ComponentId, capacity: u32, subscribers: Vec<ComponentId>, expected_item_type: String) -> Self {
        Self {
            component_id,
            capacity,
            items: VecDeque::new(),
            subscribers,
            is_full: false,
            was_empty: true,
            expected_item_type,
        }
    }

    pub fn current_count(&self) -> u32 {
        self.items.len() as u32
    }

    pub fn available_space(&self) -> u32 {
        self.capacity - self.current_count()
    }

    pub fn add_item(&mut self, item: BufferItem) -> bool {
        if self.current_count() >= self.capacity {
            return false;
        }
        
        self.was_empty = self.items.is_empty();
        self.items.push_back(item);
        self.is_full = self.current_count() >= self.capacity;
        
        true
    }

    pub fn remove_item(&mut self) -> Option<BufferItem> {
        let was_full = self.is_full;
        let item = self.items.pop_front();
        
        if item.is_some() {
            self.is_full = false;
            self.was_empty = self.items.is_empty();
        }
        
        item
    }

    pub fn create_item_added_event(&self, item: &BufferItem, timestamp: SimulationTime) -> Box<dyn Event> {
        Box::new(ItemAddedEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            timestamp,
            current_count: self.current_count(),
            item_type: item.item_type.clone(),
            item_id: item.item_id.clone(),
        })
    }

    pub fn create_buffer_full_event(&self, target_id: ComponentId, timestamp: SimulationTime) -> Box<dyn Event> {
        Box::new(BufferFullEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id,
            timestamp,
            capacity: self.capacity,
        })
    }

    pub fn create_buffer_space_available_event(&self, target_id: ComponentId, timestamp: SimulationTime) -> Box<dyn Event> {
        Box::new(BufferSpaceAvailableEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id,
            timestamp,
            available_space: self.available_space(),
        })
    }

    pub fn handle_item_ready_event(&mut self, event: &dyn Event, current_time: SimulationTime) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        let data = event.data();
        
        let item_id = match data.get("meat_id").or(data.get("bread_id")).or(data.get("burger_id")) {
            Some(ComponentValue::String(id)) => id.clone(),
            _ => Uuid::new_v4().to_string(),
        };

        let item = BufferItem {
            item_type: self.expected_item_type.clone(),
            item_id,
            timestamp: current_time,
        };

        if self.add_item(item.clone()) {
            let item_added_event = self.create_item_added_event(&item, current_time);
            output_events.push((item_added_event, 0));

            if self.is_full && !self.was_empty {
                for subscriber in &self.subscribers {
                    let buffer_full_event = self.create_buffer_full_event(subscriber.clone(), current_time);
                    output_events.push((buffer_full_event, 0));
                }
            }
        }

        output_events
    }

    pub fn handle_request_event(&mut self, event: &dyn Event, current_time: SimulationTime) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        let data = event.data();
        
        let requested_quantity = match data.get("quantity").or(data.get("burger_count")) {
            Some(ComponentValue::Int(q)) => *q as u32,
            _ => 1,
        };

        let was_full_before = self.is_full;
        let mut fulfilled_count = 0;

        for _ in 0..requested_quantity {
            if self.remove_item().is_some() {
                fulfilled_count += 1;
            } else {
                break;
            }
        }

        if fulfilled_count > 0 && was_full_before {
            for subscriber in &self.subscribers {
                let space_available_event = self.create_buffer_space_available_event(subscriber.clone(), current_time);
                output_events.push((space_available_event, 0));
            }
        }

        output_events
    }
}

/// FIFO buffer for fried meat patties
pub struct FriedMeatBuffer {
    buffer: GenericFifoBuffer,
}

impl FriedMeatBuffer {
    pub fn new(component_id: ComponentId, capacity: u32, subscribers: Vec<ComponentId>) -> Self {
        Self {
            buffer: GenericFifoBuffer::new(component_id, capacity, subscribers, "meat".to_string()),
        }
    }
}

impl BaseComponent for FriedMeatBuffer {
    fn component_id(&self) -> &ComponentId {
        &self.buffer.component_id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &[
            MEAT_READY_EVENT,
            REQUEST_ITEM_EVENT,
        ]
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        let current_time = 0;

        for event in events {
            match event.event_type() {
                MEAT_READY_EVENT => {
                    let mut new_events = self.buffer.handle_item_ready_event(event.as_ref(), current_time);
                    output_events.append(&mut new_events);
                }
                REQUEST_ITEM_EVENT => {
                    let mut new_events = self.buffer.handle_request_event(event.as_ref(), current_time);
                    output_events.append(&mut new_events);
                }
                _ => {}
            }
        }

        output_events
    }
}

/// FIFO buffer for cooked bread buns
pub struct CookedBreadBuffer {
    buffer: GenericFifoBuffer,
}

impl CookedBreadBuffer {
    pub fn new(component_id: ComponentId, capacity: u32, subscribers: Vec<ComponentId>) -> Self {
        Self {
            buffer: GenericFifoBuffer::new(component_id, capacity, subscribers, "bread".to_string()),
        }
    }
}

impl BaseComponent for CookedBreadBuffer {
    fn component_id(&self) -> &ComponentId {
        &self.buffer.component_id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &[
            BREAD_READY_EVENT,
            REQUEST_ITEM_EVENT,
        ]
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        let current_time = 0;

        for event in events {
            match event.event_type() {
                BREAD_READY_EVENT => {
                    let mut new_events = self.buffer.handle_item_ready_event(event.as_ref(), current_time);
                    output_events.append(&mut new_events);
                }
                REQUEST_ITEM_EVENT => {
                    let mut new_events = self.buffer.handle_request_event(event.as_ref(), current_time);
                    output_events.append(&mut new_events);
                }
                _ => {}
            }
        }

        output_events
    }
}

/// FIFO buffer for assembled burgers
pub struct AssemblyBuffer {
    buffer: GenericFifoBuffer,
}

impl AssemblyBuffer {
    pub fn new(component_id: ComponentId, capacity: u32, subscribers: Vec<ComponentId>) -> Self {
        Self {
            buffer: GenericFifoBuffer::new(component_id, capacity, subscribers, "burger".to_string()),
        }
    }
}

impl BaseComponent for AssemblyBuffer {
    fn component_id(&self) -> &ComponentId {
        &self.buffer.component_id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &[
            BURGER_READY_EVENT,
            PLACE_ORDER_EVENT,
        ]
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        let current_time = 0;

        for event in events {
            match event.event_type() {
                BURGER_READY_EVENT => {
                    let mut new_events = self.buffer.handle_item_ready_event(event.as_ref(), current_time);
                    output_events.append(&mut new_events);
                }
                PLACE_ORDER_EVENT => {
                    let mut new_events = self.buffer.handle_request_event(event.as_ref(), current_time);
                    output_events.append(&mut new_events);
                }
                _ => {}
            }
        }

        output_events
    }
}