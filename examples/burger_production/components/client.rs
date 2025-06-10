use rsim::core::component::BaseComponent;
use rsim::core::event::Event;
use rsim::core::types::{ComponentId, ComponentValue};
use uuid::Uuid;
use rand::prelude::*;
use rand_distr::Normal;
use log::{info, debug, warn};

use crate::events::{
    GenerateOrderEvent, PlaceOrderEvent,
    GENERATE_ORDER_EVENT,
    ITEM_ADDED_EVENT,
    BUFFER_FULL_EVENT, BUFFER_SPACE_AVAILABLE_EVENT
};

/// Client component that generates burger orders with normal distribution
#[derive(Debug)]
pub struct Client {
    pub component_id: ComponentId,
    pub assembly_buffer_id: ComponentId,
    pub order_generation_interval: u64,
    pub orders_per_generation_mean: f64,
    pub orders_per_generation_std_dev: f64,
    pub pending_orders: u32,
    pub fulfilled_orders: u32,
    pub total_orders_generated: u32,
    pub rng: StdRng,
    pub is_order_generation_stopped: bool,
}

impl Client {
    pub fn new(
        component_id: ComponentId,
        assembly_buffer_id: ComponentId,
        order_generation_interval: u64,
        orders_per_generation_mean: f64,
        orders_per_generation_std_dev: f64,
        seed: u64,
    ) -> Self {
        Self {
            component_id,
            assembly_buffer_id,
            order_generation_interval,
            orders_per_generation_mean,
            orders_per_generation_std_dev,
            pending_orders: 0,
            fulfilled_orders: 0,
            total_orders_generated: 0,
            rng: StdRng::seed_from_u64(seed),
            is_order_generation_stopped: false,
        }
    }

    /// Creates a self-scheduled GenerateOrderEvent for continuous order generation
    fn create_generate_order_event(&self) -> Box<dyn Event> {
        Box::new(GenerateOrderEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
        })
    }

    /// Creates a PlaceOrderEvent to send to the assembly buffer
    fn create_place_order_event(&self, burger_count: u32, order_id: String) -> Box<dyn Event> {
        Box::new(PlaceOrderEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id: self.assembly_buffer_id.clone(),
            burger_count,
            order_id,
        })
    }

    /// Generates a random number of orders using normal distribution
    fn generate_order_count(&mut self) -> u32 {
        let normal = Normal::new(self.orders_per_generation_mean, self.orders_per_generation_std_dev)
            .unwrap_or_else(|_| Normal::new(2.0, 1.0).unwrap());
        
        let count = normal.sample(&mut self.rng);
        // Ensure at least 1 order is generated
        (count.round() as u32).max(1)
    }

    /// Handles generating a new order
    fn handle_generate_order(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        // Only generate orders if not stopped
        if !self.is_order_generation_stopped {
            // Generate number of orders to create (each order is 1 burger)
            let order_count = self.generate_order_count();
            
            info!("[Client:{}] Generating {} individual orders (each for 1 burger)", 
                  self.component_id, order_count);

            // Create multiple individual orders, each for 1 burger
            for i in 0..order_count {
                // Check if we should stop due to buffer becoming full
                if self.is_order_generation_stopped {
                    warn!("[Client:{}] Disposed {} remaining orders due to full buffer", 
                          self.component_id, order_count - i);
                    break;
                }
                
                let order_id = format!("order_{}_{}", self.total_orders_generated + 1, i + 1);
                
                // Update statistics (each order is 1 burger)
                self.total_orders_generated += 1;
                self.pending_orders += 1;
                
                // Place the order with the assembly buffer (always 1 burger)
                let place_order_event = self.create_place_order_event(1, order_id.clone());
                output_events.push((place_order_event, 0)); // Place order immediately
                
                debug!("[Client:{}] Placed {} for 1 burger (total orders: {}, pending: {})", 
                       self.component_id, order_id, self.total_orders_generated, self.pending_orders);
            }
        } else {
            debug!("[Client:{}] Skipping order generation - order buffer is full", self.component_id);
        }

        // Always schedule next order generation check
        let next_generate_event = self.create_generate_order_event();
        output_events.push((next_generate_event, self.order_generation_interval));
        
        debug!("[Client:{}] Scheduled next order generation check in {} cycles", 
               self.component_id, self.order_generation_interval);

        output_events
    }

    /// Handles when an item (burger) is added to the assembly buffer
    fn handle_item_added(&mut self, event: &dyn Event) -> Vec<(Box<dyn Event>, u64)> {
        // Check if this is a burger item from our target buffer
        if let Some(target_ids) = event.target_ids() {
            if target_ids.contains(&self.component_id) {
                let data = event.data();
                if let Some(ComponentValue::String(item_type)) = data.get("item_type") {
                    if item_type == "burger" {
                        // A burger was fulfilled (1 burger = 1 order)
                        if self.pending_orders > 0 {
                            self.pending_orders -= 1;
                            self.fulfilled_orders += 1;
                            
                            info!("[Client:{}] Order fulfilled! (pending orders: {}, fulfilled orders: {}, total orders: {})", 
                                  self.component_id, self.pending_orders, self.fulfilled_orders, self.total_orders_generated);
                        }
                    }
                }
            }
        }

        Vec::new() // No events to generate
    }

    /// Handles when the order buffer becomes full
    fn handle_buffer_full(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        // Stop order generation when order buffer is full (backpressure)
        warn!("[Client:{}] Order buffer full - stopping order generation", self.component_id);
        self.is_order_generation_stopped = true;
        Vec::new()
    }

    /// Handles when the order buffer has space available
    fn handle_buffer_space_available(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        // Resume order generation when space becomes available
        if self.is_order_generation_stopped {
            info!("[Client:{}] Order buffer has space - resuming order generation", self.component_id);
            self.is_order_generation_stopped = false;
        }
        Vec::new()
    }

    /// Get current order statistics
    #[allow(dead_code)]
    pub fn get_statistics(&self) -> (u32, u32, u32) {
        (self.total_orders_generated, self.pending_orders, self.fulfilled_orders)
    }
}

impl BaseComponent for Client {
    fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &[
            GENERATE_ORDER_EVENT,
            ITEM_ADDED_EVENT,
            BUFFER_FULL_EVENT,
            BUFFER_SPACE_AVAILABLE_EVENT,
        ]
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        for event in events {
            match event.event_type() {
                GENERATE_ORDER_EVENT => {
                    // Only handle our own generate order events
                    if event.source_id() == &self.component_id {
                        let mut new_events = self.handle_generate_order();
                        output_events.append(&mut new_events);
                    }
                }
                ITEM_ADDED_EVENT => {
                    let mut new_events = self.handle_item_added(event.as_ref());
                    output_events.append(&mut new_events);
                }
                BUFFER_FULL_EVENT => {
                    // Only handle if this event is targeted at us (from order buffer)
                    if let Some(target_ids) = event.target_ids() {
                        if target_ids.contains(&self.component_id) {
                            let mut new_events = self.handle_buffer_full();
                            output_events.append(&mut new_events);
                        }
                    }
                }
                BUFFER_SPACE_AVAILABLE_EVENT => {
                    // Only handle if this event is targeted at us (from order buffer)
                    if let Some(target_ids) = event.target_ids() {
                        if target_ids.contains(&self.component_id) {
                            let mut new_events = self.handle_buffer_space_available();
                            output_events.append(&mut new_events);
                        }
                    }
                }
                _ => {
                    // Ignore unknown events
                }
            }
        }

        output_events
    }
}