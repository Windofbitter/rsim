//! The client component for the burger production simulation.

use rsim::core::{
    component::BaseComponent,
    event::{Event, EventId, EventType},
    types::{ComponentId, ComponentValue},
};
use std::collections::HashMap;
use uuid::Uuid;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::events::{
    GenerateOrderEvent, PlaceOrderEvent, OrderFulfilledEvent, ItemAddedEvent,
    RequestItemEvent, ItemDispatchedEvent,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ProductionMode {
    BufferBased,
    OrderBased,
}

/// Represents a pending order
#[derive(Debug, Clone)]
struct PendingOrder {
    order_id: String,
    original_quantity: u32,
    remaining_quantity: u32,
    created_at: u64,
}

/// Order statistics
#[derive(Debug, Clone)]
struct OrderStats {
    total_generated: u32,
    total_fulfilled: u32,
    total_burgers_received: u32,
}

/// The Client component, responsible for generating customer orders and consuming burgers.
pub struct Client {
    id: ComponentId,
    production_mode: ProductionMode,
    order_interval: u64,
    max_order_quantity: u32,
    min_order_quantity: u32,
    assembly_buffer_id: ComponentId,
    subscriptions: Vec<&'static str>,
    
    // Order management
    pending_order: Option<PendingOrder>,
    outstanding_requests: u32,
    
    // Statistics
    stats: OrderStats,
    
    // RNG for reproducible simulations
    rng: StdRng,
    
    // Simulation time tracking
    current_time: u64,
}

impl Client {
    /// Creates a new `Client`.
    pub fn new(
        id: ComponentId,
        production_mode: ProductionMode,
        order_interval: u64,
        min_order_quantity: u32,
        max_order_quantity: u32,
        assembly_buffer_id: ComponentId,
        seed: u64,
    ) -> Self {
        Self {
            id,
            production_mode,
            order_interval,
            max_order_quantity,
            min_order_quantity,
            assembly_buffer_id,
            subscriptions: vec![
                "GenerateOrderEvent",
                "ItemAddedEvent",
                "ItemDispatchedEvent",
                "OrderFulfilledEvent",
            ],
            pending_order: None,
            outstanding_requests: 0,
            stats: OrderStats {
                total_generated: 0,
                total_fulfilled: 0,
                total_burgers_received: 0,
            },
            rng: StdRng::seed_from_u64(seed),
            current_time: 0,
        }
    }

    /// Handle order generation
    fn handle_generate_order(&mut self, current_time: u64) -> Vec<(Box<dyn Event>, u64)> {
        let mut new_events = Vec::new();
        
        // Only generate a new order if no order is pending
        if self.pending_order.is_some() {
            log::warn!("[Client {}] Cannot generate new order - order already pending", self.id);
            return new_events;
        }

        // Generate random order quantity
        let quantity = if self.min_order_quantity == self.max_order_quantity {
            self.min_order_quantity
        } else {
            self.rng.gen_range(self.min_order_quantity..=self.max_order_quantity)
        };

        let order_id = format!("order_{}", Uuid::new_v4());
        
        // Create pending order
        self.pending_order = Some(PendingOrder {
            order_id: order_id.clone(),
            original_quantity: quantity,
            remaining_quantity: quantity,
            created_at: current_time,
        });
        
        self.stats.total_generated += 1;
        
        log::info!(
            "[Client {}] Generated order {} for {} burgers at time {}",
            self.id,
            order_id,
            quantity,
            current_time
        );

        // In OrderBased mode, send PlaceOrderEvent to trigger production
        if self.production_mode == ProductionMode::OrderBased {
            let place_order_event = PlaceOrderEvent::new(
                self.id.clone(),
                None, // Broadcast to all components
                order_id.clone(),
                quantity as i32,
            );
            new_events.push((Box::new(place_order_event), 0));
            log::info!("[Client {}] Sent PlaceOrderEvent for {} items", self.id, quantity);
        }

        new_events
    }

    /// Handle item added notification from assembly buffer
    fn handle_item_added(&mut self, event: &dyn Event) -> Vec<(Box<dyn Event>, u64)> {
        let mut new_events = Vec::new();
        
        // Only process if we have a pending order
        if self.pending_order.is_none() {
            return new_events;
        }

        let data = event.data();
        let buffer_type = data.get("buffer_type")
            .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("unknown");

        // Only process items from assembly buffer (burgers)
        if buffer_type != "assembly" {
            return new_events;
        }

        // Check if we need more items
        if let Some(ref order) = self.pending_order {
            let items_needed = order.remaining_quantity;
            
            // Only request if we haven't already requested all items we need
            if self.outstanding_requests >= items_needed {
                log::info!(
                    "[Client {}] Already have {} outstanding requests for {} remaining items, skipping",
                    self.id,
                    self.outstanding_requests,
                    items_needed
                );
                return new_events;
            }

            log::info!(
                "[Client {}] Burger available in assembly buffer, requesting item ({} outstanding, {} needed)",
                self.id,
                self.outstanding_requests,
                items_needed
            );

            // Send request for burger
            let request_event = RequestItemEvent::new(
                self.id.clone(),
                Some(vec![self.assembly_buffer_id.clone()]),
                self.id.clone(),
                "burger".to_string(),
            );
            new_events.push((Box::new(request_event), 0));
            
            // Increment outstanding requests
            self.outstanding_requests += 1;
        }

        new_events
    }

    /// Handle item dispatched from assembly buffer
    fn handle_item_dispatched(&mut self, event: &dyn Event) -> Vec<(Box<dyn Event>, u64)> {
        let mut new_events = Vec::new();
        
        let data = event.data();
        let item_type = data.get("item_type")
            .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("unknown");
        let item_id = data.get("item_id")
            .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("unknown");
        let success = data.get("success")
            .and_then(|v| if let ComponentValue::Bool(b) = v { Some(*b) } else { None })
            .unwrap_or(false);

        // Only process burger items
        if item_type != "burger" {
            return new_events;
        }

        if !success {
            log::warn!("[Client {}] Failed to receive burger {}", self.id, item_id);
            return new_events;
        }

        // Update pending order and statistics
        if let Some(ref mut order) = self.pending_order {
            if order.remaining_quantity > 0 {
                order.remaining_quantity -= 1;
                self.stats.total_burgers_received += 1;
                
                // Decrement outstanding requests since we received an item
                if self.outstanding_requests > 0 {
                    self.outstanding_requests -= 1;
                }
                
                log::info!(
                    "[Client {}] Received burger {}. Order {} now has {}/{} burgers remaining ({} requests outstanding)",
                    self.id,
                    item_id,
                    order.order_id,
                    order.remaining_quantity,
                    order.original_quantity,
                    self.outstanding_requests
                );

                // Check if order is complete
                if order.remaining_quantity == 0 {
                    let fulfillment_time = self.current_time - order.created_at;
                    let order_id = order.order_id.clone();
                    
                    // Generate OrderFulfilledEvent
                    let fulfilled_event = OrderFulfilledEvent::new(
                        self.id.clone(),
                        Some(vec![self.id.clone()]), // Self-targeted
                        order_id.clone(),
                        fulfillment_time,
                    );
                    new_events.push((Box::new(fulfilled_event), 0));
                    
                    log::info!(
                        "[Client {}] Order {} fulfilled in {} cycles",
                        self.id,
                        order_id,
                        fulfillment_time
                    );
                }
            }
        }

        new_events
    }

    /// Handle order fulfilled
    fn handle_order_fulfilled(&mut self, event: &dyn Event) -> Vec<(Box<dyn Event>, u64)> {
        let mut new_events = Vec::new();
        
        let data = event.data();
        let order_id = data.get("order_id")
            .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("unknown");

        // Verify this is our order
        if let Some(ref pending) = self.pending_order {
            if pending.order_id == order_id {
                // Clear pending order
                self.pending_order = None;
                self.stats.total_fulfilled += 1;
                
                // Reset outstanding requests
                self.outstanding_requests = 0;
                
                log::info!("[Client {}] Order {} completed and cleared", self.id, order_id);
                
                // Schedule next order generation
                let next_order_event = GenerateOrderEvent::new(
                    self.id.clone(),
                    Some(vec![self.id.clone()]),
                );
                new_events.push((Box::new(next_order_event), self.order_interval));
                
                log::info!("[Client {}] Scheduled next order generation in {} cycles", self.id, self.order_interval);
            }
        }

        new_events
    }

    /// Get current statistics
    pub fn get_stats(&self) -> (u32, u32, u32) {
        (self.stats.total_generated, self.stats.total_fulfilled, self.stats.total_burgers_received)
    }

    /// Check if client has a pending order
    pub fn has_pending_order(&self) -> bool {
        self.pending_order.is_some()
    }

    /// Get pending order info
    pub fn get_pending_order_info(&self) -> Option<(String, u32, u32)> {
        self.pending_order.as_ref().map(|order| {
            (order.order_id.clone(), order.original_quantity, order.remaining_quantity)
        })
    }
}

impl BaseComponent for Client {
    fn component_id(&self) -> &ComponentId {
        &self.id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &self.subscriptions
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut new_events = Vec::new();

        for event in events {
            // Update current time based on event timing (approximation)
            self.current_time += 1;

            match event.event_type() {
                "GenerateOrderEvent" => {
                    // Check if this event is for us
                    if let Some(target_ids) = event.target_ids() {
                        if !target_ids.contains(&self.id) {
                            continue;
                        }
                    }
                    let mut order_events = self.handle_generate_order(self.current_time);
                    new_events.append(&mut order_events);
                }
                "ItemAddedEvent" => {
                    let mut item_events = self.handle_item_added(event.as_ref());
                    new_events.append(&mut item_events);
                }
                "ItemDispatchedEvent" => {
                    // Check if this event is for us
                    if let Some(target_ids) = event.target_ids() {
                        if !target_ids.contains(&self.id) {
                            continue;
                        }
                    }
                    let mut dispatch_events = self.handle_item_dispatched(event.as_ref());
                    new_events.append(&mut dispatch_events);
                }
                "OrderFulfilledEvent" => {
                    // Check if this event is for us
                    if let Some(target_ids) = event.target_ids() {
                        if !target_ids.contains(&self.id) {
                            continue;
                        }
                    }
                    let mut fulfilled_events = self.handle_order_fulfilled(event.as_ref());
                    new_events.append(&mut fulfilled_events);
                }
                _ => {
                    log::warn!(
                        "[Client {}] received unhandled event type: {}",
                        self.id,
                        event.event_type()
                    );
                }
            }
        }

        new_events
    }
}