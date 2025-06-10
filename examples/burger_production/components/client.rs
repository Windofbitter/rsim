use rsim::core::component::BaseComponent;
use rsim::core::event::Event;
use rsim::core::types::{ComponentId, ComponentValue, SimulationTime};
use uuid::Uuid;
use rand::prelude::*;
use rand_distr::Normal;

use crate::events::{
    GenerateOrderEvent, PlaceOrderEvent,
    GENERATE_ORDER_EVENT,
    ITEM_ADDED_EVENT
};

/// Client component that generates burger orders with normal distribution
#[derive(Debug)]
pub struct Client {
    pub component_id: ComponentId,
    pub assembly_buffer_id: ComponentId,
    pub order_generation_interval: u64,
    pub order_size_mean: f64,
    pub order_size_std_dev: f64,
    pub pending_orders: u32,
    pub fulfilled_orders: u32,
    pub total_orders_generated: u32,
    pub rng: StdRng,
}

impl Client {
    pub fn new(
        component_id: ComponentId,
        assembly_buffer_id: ComponentId,
        order_generation_interval: u64,
        order_size_mean: f64,
        order_size_std_dev: f64,
        seed: u64,
    ) -> Self {
        Self {
            component_id,
            assembly_buffer_id,
            order_generation_interval,
            order_size_mean,
            order_size_std_dev,
            pending_orders: 0,
            fulfilled_orders: 0,
            total_orders_generated: 0,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Creates a self-scheduled GenerateOrderEvent for continuous order generation
    fn create_generate_order_event(&self, timestamp: SimulationTime) -> Box<dyn Event> {
        Box::new(GenerateOrderEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            timestamp,
        })
    }

    /// Creates a PlaceOrderEvent to send to the assembly buffer
    fn create_place_order_event(&self, timestamp: SimulationTime, burger_count: u32, order_id: String) -> Box<dyn Event> {
        Box::new(PlaceOrderEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id: self.assembly_buffer_id.clone(),
            timestamp,
            burger_count,
            order_id,
        })
    }

    /// Generates a random order size using normal distribution
    fn generate_order_size(&mut self) -> u32 {
        let normal = Normal::new(self.order_size_mean, self.order_size_std_dev)
            .unwrap_or_else(|_| Normal::new(1.0, 0.5).unwrap());
        
        let size = normal.sample(&mut self.rng);
        // Ensure order size is at least 1
        (size.round() as u32).max(1)
    }

    /// Handles generating a new order
    fn handle_generate_order(&mut self, current_time: SimulationTime) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        // Generate order size with normal distribution
        let burger_count = self.generate_order_size();
        let order_id = format!("order_{}", self.total_orders_generated + 1);

        // Update statistics
        self.total_orders_generated += 1;
        self.pending_orders += burger_count;

        // Place the order with the assembly buffer
        let place_order_event = self.create_place_order_event(current_time, burger_count, order_id);
        output_events.push((place_order_event, 0)); // Place order immediately

        // Schedule next order generation
        let next_generate_event = self.create_generate_order_event(current_time);
        output_events.push((next_generate_event, self.order_generation_interval));

        output_events
    }

    /// Handles when an item (burger) is added to the assembly buffer
    fn handle_item_added(&mut self, event: &dyn Event, _current_time: SimulationTime) -> Vec<(Box<dyn Event>, u64)> {
        // Check if this is a burger item from our target buffer
        if let Some(target_ids) = event.target_ids() {
            if target_ids.contains(&self.component_id) {
                let data = event.data();
                if let Some(ComponentValue::String(item_type)) = data.get("item_type") {
                    if item_type == "burger" {
                        // A burger was fulfilled
                        if self.pending_orders > 0 {
                            self.pending_orders -= 1;
                            self.fulfilled_orders += 1;
                        }
                    }
                }
            }
        }

        Vec::new() // No events to generate
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
        ]
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        let current_time = 0; // In a real simulation, this would come from the engine

        for event in events {
            match event.event_type() {
                GENERATE_ORDER_EVENT => {
                    // Only handle our own generate order events
                    if event.source_id() == &self.component_id {
                        let mut new_events = self.handle_generate_order(current_time);
                        output_events.append(&mut new_events);
                    }
                }
                ITEM_ADDED_EVENT => {
                    let mut new_events = self.handle_item_added(event.as_ref(), current_time);
                    output_events.append(&mut new_events);
                }
                _ => {
                    // Ignore unknown events
                }
            }
        }

        output_events
    }
}