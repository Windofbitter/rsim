use rsim::core::component::BaseComponent;
use rsim::core::event::Event;
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::VecDeque;
use uuid::Uuid;
use log::{info, debug, warn};

use crate::events::{
    ItemAddedEvent, BufferFullEvent, BufferSpaceAvailableEvent,
    PartialOrderFulfillmentEvent, OrderCompletedEvent,
    MEAT_READY_EVENT, BREAD_READY_EVENT, BURGER_READY_EVENT, PLACE_ORDER_EVENT, REQUEST_ITEM_EVENT,
    ProductionRequestEvent, PRODUCTION_REQUEST_EVENT, ITEM_ADDED_EVENT,
    PARTIAL_ORDER_FULFILLMENT_EVENT, ORDER_COMPLETED_EVENT
};

#[derive(Debug, Clone)]
pub struct BufferItem {
    pub item_type: String,
    pub item_id: String,
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
        let _was_full = self.is_full;
        let item = self.items.pop_front();
        
        if item.is_some() {
            self.is_full = false;
            self.was_empty = self.items.is_empty();
        }
        
        item
    }

    pub fn create_item_added_event(&self, item: &BufferItem) -> Box<dyn Event> {
        Box::new(ItemAddedEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            current_count: self.current_count(),
            item_type: item.item_type.clone(),
            item_id: item.item_id.clone(),
        })
    }

    pub fn create_buffer_full_event(&self, target_id: ComponentId) -> Box<dyn Event> {
        Box::new(BufferFullEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id,
            capacity: self.capacity,
        })
    }

    pub fn create_buffer_space_available_event(&self, target_id: ComponentId) -> Box<dyn Event> {
        Box::new(BufferSpaceAvailableEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id,
            available_space: self.available_space(),
        })
    }

    pub fn handle_item_ready_event(&mut self, event: &dyn Event) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        let data = event.data();
        
        let item_id = match data.get("meat_id").or(data.get("bread_id")).or(data.get("burger_id")) {
            Some(ComponentValue::String(id)) => id.clone(),
            _ => Uuid::new_v4().to_string(),
        };

        let item = BufferItem {
            item_type: self.expected_item_type.clone(),
            item_id: item_id.clone(),
        };

        if self.add_item(item.clone()) {
            info!("[Buffer:{}] Added {} {} (count: {}/{})", 
                  self.component_id, self.expected_item_type, item_id, 
                  self.current_count(), self.capacity);
                  
            let item_added_event = self.create_item_added_event(&item);
            output_events.push((item_added_event, 0));

            if self.is_full && !self.was_empty {
                warn!("[Buffer:{}] Buffer is now full - notifying {} subscribers", 
                      self.component_id, self.subscribers.len());
                for subscriber in &self.subscribers {
                    let buffer_full_event = self.create_buffer_full_event(subscriber.clone());
                    output_events.push((buffer_full_event, 0));
                }
            }
        } else {
            warn!("[Buffer:{}] Failed to add {} {} - buffer full ({}/{})", 
                  self.component_id, self.expected_item_type, item_id, 
                  self.current_count(), self.capacity);
        }

        output_events
    }

    pub fn handle_request_event(&mut self, event: &dyn Event) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        let data = event.data();
        
        let requested_quantity = match data.get("quantity").or(data.get("burger_count")) {
            Some(ComponentValue::Int(q)) => *q as u32,
            _ => 1,
        };
        
        debug!("[Buffer:{}] Received request for {} {} items from {}", 
               self.component_id, requested_quantity, self.expected_item_type, event.source_id());

        let was_full_before = self.is_full;
        let mut fulfilled_count = 0;

        for _ in 0..requested_quantity {
            if let Some(item) = self.remove_item() {
                fulfilled_count += 1;
                debug!("[Buffer:{}] Fulfilled request for {} {} (remaining: {}/{})", 
                       self.component_id, self.expected_item_type, item.item_id, 
                       self.current_count(), self.capacity);
            } else {
                break;
            }
        }
        
        if fulfilled_count > 0 {
            info!("[Buffer:{}] Fulfilled {}/{} requested {} items", 
                  self.component_id, fulfilled_count, requested_quantity, self.expected_item_type);
        } else {
            debug!("[Buffer:{}] Could not fulfill any requests - buffer empty", self.component_id);
        }

        if fulfilled_count > 0 && was_full_before {
            info!("[Buffer:{}] Buffer no longer full - notifying {} subscribers of available space", 
                  self.component_id, self.subscribers.len());
            for subscriber in &self.subscribers {
                let space_available_event = self.create_buffer_space_available_event(subscriber.clone());
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

        for event in events {
            match event.event_type() {
                MEAT_READY_EVENT => {
                    let mut new_events = self.buffer.handle_item_ready_event(event.as_ref());
                    output_events.append(&mut new_events);
                }
                REQUEST_ITEM_EVENT => {
                    let mut new_events = self.buffer.handle_request_event(event.as_ref());
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

        for event in events {
            match event.event_type() {
                BREAD_READY_EVENT => {
                    let mut new_events = self.buffer.handle_item_ready_event(event.as_ref());
                    output_events.append(&mut new_events);
                }
                REQUEST_ITEM_EVENT => {
                    let mut new_events = self.buffer.handle_request_event(event.as_ref());
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

        for event in events {
            match event.event_type() {
                BURGER_READY_EVENT => {
                    let mut new_events = self.buffer.handle_item_ready_event(event.as_ref());
                    output_events.append(&mut new_events);
                }
                PLACE_ORDER_EVENT => {
                    let mut new_events = self.buffer.handle_request_event(event.as_ref());
                    output_events.append(&mut new_events);
                }
                _ => {}
            }
        }

        output_events
    }
}

#[derive(Debug, Clone)]
pub struct OrderItem {
    pub order_id: String,
    pub burger_count: u32,
    pub burgers_fulfilled: u32,
    pub client_id: ComponentId,
}

impl OrderItem {
    pub fn remaining_burgers(&self) -> u32 {
        self.burger_count - self.burgers_fulfilled
    }
    
    pub fn is_complete(&self) -> bool {
        self.burgers_fulfilled >= self.burger_count
    }
}

/// OrderBuffer - FIFO queue for pending orders between Client and production system
#[derive(Debug)]
pub struct OrderBuffer {
    pub component_id: ComponentId,
    pub capacity: u32,
    pub orders: VecDeque<OrderItem>,
    pub assembler_id: ComponentId,
    pub assembly_buffer_id: ComponentId,
    pub subscribers: Vec<ComponentId>,
    pub is_full: bool,
    pub cached_assembly_buffer_count: u32,
}

impl OrderBuffer {
    pub fn new(
        component_id: ComponentId,
        capacity: u32,
        assembler_id: ComponentId,
        assembly_buffer_id: ComponentId,
        subscribers: Vec<ComponentId>,
    ) -> Self {
        Self {
            component_id,
            capacity,
            orders: VecDeque::new(),
            assembler_id,
            assembly_buffer_id,
            subscribers,
            is_full: false,
            cached_assembly_buffer_count: 0,
        }
    }

    pub fn current_count(&self) -> u32 {
        self.orders.len() as u32
    }

    pub fn available_space(&self) -> u32 {
        self.capacity - self.current_count()
    }

    pub fn add_order(&mut self, order: OrderItem) -> bool {
        if self.current_count() >= self.capacity {
            warn!("OrderBuffer {} is full, rejecting order {}", 
                  self.component_id, order.order_id);
            return false;
        }
        
        info!("OrderBuffer {} queued order {} for {} burgers", 
              self.component_id, order.order_id, order.burger_count);
        self.orders.push_back(order);
        true
    }

    pub fn get_next_order(&mut self) -> Option<OrderItem> {
        let order = self.orders.pop_front();
        if let Some(ref order) = order {
            info!("OrderBuffer {} processing order {} for {} burgers", 
                  self.component_id, order.order_id, order.burger_count);
        }
        order
    }
    
    fn create_buffer_full_event(&self, target_id: ComponentId) -> Box<dyn Event> {
        Box::new(BufferFullEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id,
            capacity: self.capacity,
        })
    }

    fn create_buffer_space_available_event(&self, target_id: ComponentId) -> Box<dyn Event> {
        Box::new(BufferSpaceAvailableEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id,
            available_space: self.available_space(),
        })
    }

    fn handle_item_added_event(&mut self, event: &dyn Event) -> Vec<(Box<dyn Event>, u64)> {
        // Check if this is a burger from AssemblyBuffer
        if event.source_id() == &self.assembly_buffer_id {
            let data = event.data();
            if let Some(ComponentValue::String(item_type)) = data.get("item_type") {
                if item_type == "burger" {
                    // Update cached count from the event
                    if let Some(ComponentValue::Int(current_count)) = data.get("current_count") {
                        self.cached_assembly_buffer_count = *current_count as u32;
                    }
                    
                    debug!("[OrderBuffer:{}] Burger added to AssemblyBuffer (count: {}), trying to fulfill orders", 
                           self.component_id, self.cached_assembly_buffer_count);
                    return self.try_fulfill_orders();
                }
            }
        }
        Vec::new()
    }

    fn try_fulfill_orders(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        
        let available_burgers = self.query_assembly_buffer_count();
        let mut remaining_burgers = available_burgers;
        let mut completed_orders = Vec::new();
        let mut updated_orders = VecDeque::new();
        
        info!("[OrderBuffer:{}] Trying to fulfill orders with {} available burgers", 
              self.component_id, available_burgers);
        
        // Process orders in FIFO order with partial fulfillment
        while let Some(mut order) = self.orders.pop_front() {
            let needed = order.remaining_burgers();
            
            if remaining_burgers >= needed {
                // Can complete this order
                order.burgers_fulfilled = order.burger_count;
                remaining_burgers -= needed;
                completed_orders.push(order.clone());
                
                info!("[OrderBuffer:{}] Order {} completed ({}/{} burgers)", 
                      self.component_id, order.order_id, order.burger_count, order.burger_count);
            } else if remaining_burgers > 0 {
                // Partial fulfillment
                order.burgers_fulfilled += remaining_burgers;
                remaining_burgers = 0;
                updated_orders.push_back(order.clone());
                
                info!("[OrderBuffer:{}] Order {} partially fulfilled ({}/{} burgers)", 
                      self.component_id, order.order_id, order.burgers_fulfilled, order.burger_count);
                
                // Put remaining orders back without processing
                while let Some(remaining_order) = self.orders.pop_front() {
                    updated_orders.push_back(remaining_order);
                }
                break;
            } else {
                // No burgers left, put order back
                updated_orders.push_back(order);
                
                // Put remaining orders back
                while let Some(remaining_order) = self.orders.pop_front() {
                    updated_orders.push_back(remaining_order);
                }
                break;
            }
        }
        
        self.orders = updated_orders;
        
        // Update cached count to reflect consumed burgers
        let burgers_consumed = available_burgers - remaining_burgers;
        self.cached_assembly_buffer_count = remaining_burgers;
        
        if burgers_consumed > 0 {
            info!("[OrderBuffer:{}] Consumed {} burgers for order fulfillment (remaining: {})", 
                  self.component_id, burgers_consumed, self.cached_assembly_buffer_count);
        }
        
        // Generate fulfillment events for completed orders
        for completed_order in completed_orders {
            let order_completed_event = Box::new(OrderCompletedEvent {
                id: Uuid::new_v4().to_string(),
                source_id: self.component_id.clone(),
                target_id: completed_order.client_id.clone(),
                order_id: completed_order.order_id.clone(),
                total_burgers: completed_order.burger_count,
            });
            output_events.push((order_completed_event, 0));
            
            debug!("[OrderBuffer:{}] Generated completion event for order {}", 
                   self.component_id, completed_order.order_id);
        }
        
        output_events
    }

    fn query_assembly_buffer_count(&self) -> u32 {
        // Return cached count from last ITEM_ADDED_EVENT
        self.cached_assembly_buffer_count
    }

    fn handle_place_order_event(&mut self, event: &dyn Event) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        
        if let Some(order_data) = self.extract_order_from_event(event) {
            let order_item = OrderItem {
                order_id: order_data.order_id,
                burger_count: order_data.burger_count,
                burgers_fulfilled: 0,
                client_id: event.source_id().clone(),
            };
            
            let was_full_before = self.is_full;
            
            if self.add_order(order_item.clone()) {
                info!("[OrderBuffer:{}] Added order {} for {} burgers to queue", 
                      self.component_id, order_item.order_id, order_item.burger_count);
                
                // Try to fulfill orders immediately if burgers are available
                let mut fulfillment_events = self.try_fulfill_orders();
                output_events.append(&mut fulfillment_events);
                
                // Check if buffer became full after adding order
                self.is_full = self.current_count() >= self.capacity;
                if self.is_full && !was_full_before {
                    warn!("[OrderBuffer:{}] Buffer is now full - notifying {} subscribers", 
                          self.component_id, self.subscribers.len());
                    for subscriber in &self.subscribers {
                        let buffer_full_event = self.create_buffer_full_event(subscriber.clone());
                        output_events.push((buffer_full_event, 0));
                    }
                }
            }
        }
        
        output_events
    }

    fn extract_order_from_event(&self, event: &dyn Event) -> Option<PlaceOrderEventData> {
        if event.event_type() == PLACE_ORDER_EVENT {
            let data = event.data();
            if let (Some(ComponentValue::Int(burger_count)), Some(ComponentValue::String(order_id))) = 
                (data.get("burger_count"), data.get("order_id")) {
                return Some(PlaceOrderEventData {
                    burger_count: *burger_count as u32,
                    order_id: order_id.clone(),
                });
            }
        }
        None
    }
}

#[derive(Debug)]
struct PlaceOrderEventData {
    burger_count: u32,
    order_id: String,
}

impl BaseComponent for OrderBuffer {
    fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &[
            PLACE_ORDER_EVENT,
            PRODUCTION_REQUEST_EVENT,  // Listen for when assembler processes orders
            ITEM_ADDED_EVENT,  // Listen for burgers added to AssemblyBuffer
        ]
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        for event in events {
            match event.event_type() {
                PLACE_ORDER_EVENT => {
                    let mut new_events = self.handle_place_order_event(event.as_ref());
                    output_events.append(&mut new_events);
                }
                ITEM_ADDED_EVENT => {
                    let mut new_events = self.handle_item_added_event(event.as_ref());
                    output_events.append(&mut new_events);
                }
                _ => {}
            }
        }

        output_events
    }
}