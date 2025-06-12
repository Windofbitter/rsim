//! The fryer component for the burger production simulation.

use rsim::core::{
    component::BaseComponent,
    event::{Event, EventId, EventType},
    types::{ComponentId, ComponentValue},
};
use std::collections::HashMap;
use uuid::Uuid;

use super::super::events::{
    MeatReadyEvent, TriggerProductionEvent, PlaceOrderEvent, ItemDroppedEvent,
};
use super::super::config::ProductionMode;

/// The Fryer component, responsible for converting raw meat into fried patties.
pub struct Fryer {
    id: ComponentId,
    production_mode: ProductionMode,
    processing_time: u64,
    is_production_stopped: bool,
    subscriptions: Vec<&'static str>,
    // OrderBased mode fields
    target_quantity: Option<u32>,
    completed_count: u32,
    // Backpressure handling
    held_item: Option<MeatReadyEvent>,
}

impl Fryer {
    /// Creates a new `Fryer`.
    pub fn new(
        id: ComponentId,
        production_mode: ProductionMode,
        processing_time: u64,
    ) -> Self {
        Self {
            id,
            production_mode,
            processing_time,
            is_production_stopped: false,
            subscriptions: vec![
                "TriggerProductionEvent",
                "BufferFullEvent",
                "BufferSpaceAvailableEvent",
                "PlaceOrderEvent",
                "ItemDroppedEvent",
            ],
            target_quantity: None,
            completed_count: 0,
            held_item: None,
        }
    }

    fn handle_trigger_production(&mut self) -> Option<MeatReadyEvent> {
        if self.is_production_stopped {
            return None;
        }

        // Check if we've completed the order in OrderBased mode
        if self.production_mode == ProductionMode::OrderBased {
            if let Some(target) = self.target_quantity {
                if self.completed_count >= target {
                    log::info!("[Fryer {}] Order complete. Produced {} items.", self.id, self.completed_count);
                    self.target_quantity = None;
                    self.completed_count = 0;
                    return None;
                }
            } else {
                // No active order in OrderBased mode
                return None;
            }
        }

        // In a real implementation, we would check for raw materials.
        // For now, we assume they are always available.
        log::info!("[Fryer {}] Starting to fry a new patty.", self.id);

        let item_id = format!("patty_{}", Uuid::new_v4());
        Some(MeatReadyEvent::new(
            self.id.clone(),
            None,
            item_id,
        ))
    }
}

impl BaseComponent for Fryer {
    fn component_id(&self) -> &ComponentId {
        &self.id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &self.subscriptions
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut new_events: Vec<(Box<dyn Event>, u64)> = Vec::new();
        
        println!("[FRYER {}] Processing {} events", self.id, events.len());

        for event in events {
            match event.event_type() {
                "TriggerProductionEvent" => {
                    // Check if we have a held item to retry first
                    if let Some(held_item) = self.held_item.take() {
                        log::info!("[Fryer {}] Retrying held item after space became available.", self.id);
                        new_events.push((Box::new(held_item), 0));
                        
                        // In OrderBased mode, re-increment completed count for the retry
                        if self.production_mode == ProductionMode::OrderBased {
                            self.completed_count += 1;
                            log::info!("[Fryer {}] Retrying item, count now {}/{}", self.id, self.completed_count, 
                                self.target_quantity.unwrap_or(0));
                        }
                    } else if let Some(meat_ready_event) = self.handle_trigger_production() {
                        new_events.push((Box::new(meat_ready_event), self.processing_time));

                        // In OrderBased mode, optimistically increment completed count
                        // It will be adjusted if we receive ItemDroppedEvent
                        if self.production_mode == ProductionMode::OrderBased {
                            self.completed_count += 1;
                            log::info!("[Fryer {}] Produced item {}/{}", self.id, self.completed_count, 
                                self.target_quantity.unwrap_or(0));
                        }

                        // In buffer-based mode, schedule the next production cycle.
                        // In order-based mode, only continue if we haven't reached target quantity
                        if self.production_mode == ProductionMode::BufferBased && !self.is_production_stopped {
                            let trigger_event =
                                TriggerProductionEvent::new(self.id.clone(), Some(vec![self.id.clone()]));
                            new_events.push((Box::new(trigger_event), 1));
                        } else if self.production_mode == ProductionMode::OrderBased {
                            if let Some(target) = self.target_quantity {
                                if self.completed_count < target {
                                    let trigger_event =
                                        TriggerProductionEvent::new(self.id.clone(), Some(vec![self.id.clone()]));
                                    new_events.push((Box::new(trigger_event), 1));
                                }
                            }
                        }
                    }
                }
                "PlaceOrderEvent" => {
                    let data = event.data();
                    if let Some(ComponentValue::Int(quantity)) = data.get("quantity") {
                        log::info!(
                            "[Fryer {}] Received PlaceOrderEvent for {} items",
                            self.id,
                            quantity
                        );
                        self.target_quantity = Some(*quantity as u32);
                        self.completed_count = 0;
                        
                        // Start production immediately in OrderBased mode
                        if self.production_mode == ProductionMode::OrderBased {
                            let trigger_event =
                                TriggerProductionEvent::new(self.id.clone(), Some(vec![self.id.clone()]));
                            new_events.push((Box::new(trigger_event), 1));
                        }
                    }
                }
                "ItemDroppedEvent" => {
                    // Check if this component is the intended recipient
                    if let Some(target_ids) = event.target_ids() {
                        if !target_ids.contains(&self.id) {
                            // This event is not for us, ignore it
                            continue;
                        }
                    }
                    
                    let data = event.data();
                    let item_type = data.get("item_type")
                        .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("unknown");
                    
                    // Only handle drops for meat items (our output)
                    if item_type != "meat" && item_type != "fried_meat" {
                        continue;
                    }
                    
                    let item_id = data.get("item_id")
                        .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("unknown");
                    let reason = data.get("reason")
                        .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("unknown");
                    
                    log::info!(
                        "[Fryer {}] Meat item {} was dropped. Reason: {}. Will retry when space available.",
                        self.id,
                        item_id,
                        reason
                    );
                    
                    // In OrderBased mode, decrement completed count since the item was rejected
                    if self.production_mode == ProductionMode::OrderBased && self.completed_count > 0 {
                        self.completed_count -= 1;
                        log::info!("[Fryer {}] Adjusted completed count to {} after drop", self.id, self.completed_count);
                    }
                    
                    // Store the item to retry later with the same item_id
                    self.held_item = Some(MeatReadyEvent::new(
                        self.id.clone(),
                        None,
                        item_id.to_string(),
                    ));
                    self.is_production_stopped = true;
                }
                "BufferFullEvent" => {
                    let data = event.data();
                    let buffer_type = data.get("buffer_type")
                        .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("unknown");
                    
                    // Only pause production if this is from our output buffer (fried_meat buffer)
                    if buffer_type == "fried_meat" {
                        log::info!(
                            "[Fryer {}] Fried meat buffer is full. Pausing production.",
                            self.id
                        );
                        self.is_production_stopped = true;
                    }
                }
                "BufferSpaceAvailableEvent" => {
                    let data = event.data();
                    let buffer_type = data.get("buffer_type")
                        .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("unknown");
                    
                    // Only resume production if this is from our output buffer (fried_meat buffer)
                    if buffer_type == "fried_meat" {
                        log::info!(
                            "[Fryer {}] Fried meat buffer has space available. Resuming production.",
                            self.id
                        );
                        let was_stopped = self.is_production_stopped;
                        self.is_production_stopped = false;

                        // If we have a held item, retry it immediately
                        if self.held_item.is_some() {
                            let trigger_event =
                                TriggerProductionEvent::new(self.id.clone(), Some(vec![self.id.clone()]));
                            new_events.push((Box::new(trigger_event), 0));
                        }
                        // Otherwise, if in buffer-based mode, trigger production if it was stopped
                        else if self.production_mode == ProductionMode::BufferBased && was_stopped {
                            let trigger_event =
                                TriggerProductionEvent::new(self.id.clone(), Some(vec![self.id.clone()]));
                            new_events.push((Box::new(trigger_event), 1));
                        }
                    }
                }
                _ => {
                    log::warn!(
                        "[Fryer {}] received unhandled event type: {}",
                        self.id,
                        event.event_type()
                    );
                }
            }
        }
        new_events
    }
} 