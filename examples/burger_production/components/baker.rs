//! The baker component for the burger production simulation.

use rsim::core::{
    component::BaseComponent,
    event::{Event, EventId, EventType},
    types::{ComponentId, ComponentValue},
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::events::{
    BreadReadyEvent, TriggerProductionEvent, PlaceOrderEvent, ItemDroppedEvent,
};
use crate::ProductionMode;

/// The Baker component, responsible for converting raw bread into cooked buns.
pub struct Baker {
    id: ComponentId,
    production_mode: ProductionMode,
    output_buffer: ComponentId,
    processing_time: u64,
    is_production_stopped: bool,
    subscriptions: Vec<&'static str>,
    // OrderBased mode fields
    target_quantity: Option<u32>,
    completed_count: u32,
    // Backpressure handling
    held_item: Option<BreadReadyEvent>,
}

impl Baker {
    /// Creates a new `Baker`.
    pub fn new(
        id: ComponentId,
        production_mode: ProductionMode,
        output_buffer: ComponentId,
        processing_time: u64,
    ) -> Self {
        Self {
            id,
            production_mode,
            output_buffer,
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

    fn handle_trigger_production(&mut self) -> Option<BreadReadyEvent> {
        if self.is_production_stopped {
            return None;
        }

        // Check if we've completed the order in OrderBased mode
        if self.production_mode == ProductionMode::OrderBased {
            if let Some(target) = self.target_quantity {
                if self.completed_count >= target {
                    log::info!("[Baker {}] Order complete. Produced {} items.", self.id, self.completed_count);
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
        log::info!("[Baker {}] Starting to bake a new bun.", self.id);

        let item_id = format!("bun_{}", Uuid::new_v4());
        Some(BreadReadyEvent::new(
            self.id.clone(),
            Some(vec![self.output_buffer.clone()]),
            item_id,
        ))
    }
}

impl BaseComponent for Baker {
    fn component_id(&self) -> &ComponentId {
        &self.id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &self.subscriptions
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut new_events = Vec::new();

        for event in events {
            match event.event_type() {
                "TriggerProductionEvent" => {
                    // Check if we have a held item to retry first
                    if let Some(held_item) = self.held_item.take() {
                        log::info!("[Baker {}] Retrying held item after space became available.", self.id);
                        new_events.push((Box::new(held_item), 0));
                        
                        // In OrderBased mode, re-increment completed count for the retry
                        if self.production_mode == ProductionMode::OrderBased {
                            self.completed_count += 1;
                            log::info!("[Baker {}] Retrying item, count now {}/{}", self.id, self.completed_count, 
                                self.target_quantity.unwrap_or(0));
                        }
                    } else if let Some(bread_ready_event) = self.handle_trigger_production() {
                        new_events.push((Box::new(bread_ready_event), self.processing_time));

                        // In OrderBased mode, optimistically increment completed count
                        // It will be adjusted if we receive ItemDroppedEvent
                        if self.production_mode == ProductionMode::OrderBased {
                            self.completed_count += 1;
                            log::info!("[Baker {}] Produced item {}/{}", self.id, self.completed_count, 
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
                            "[Baker {}] Received PlaceOrderEvent for {} items",
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
                    let data = event.data();
                    let item_id = data.get("item_id")
                        .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("unknown");
                    let reason = data.get("reason")
                        .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("unknown");
                    
                    log::info!(
                        "[Baker {}] Item {} was dropped. Reason: {}. Will retry when space available.",
                        self.id,
                        item_id,
                        reason
                    );
                    
                    // In OrderBased mode, decrement completed count since the item was rejected
                    if self.production_mode == ProductionMode::OrderBased && self.completed_count > 0 {
                        self.completed_count -= 1;
                        log::info!("[Baker {}] Adjusted completed count to {} after drop", self.id, self.completed_count);
                    }
                    
                    // Store the item to retry later with the same item_id
                    self.held_item = Some(BreadReadyEvent::new(
                        self.id.clone(),
                        Some(vec![self.output_buffer.clone()]),
                        item_id.to_string(),
                    ));
                    self.is_production_stopped = true;
                }
                "BufferFullEvent" => {
                    log::info!(
                        "[Baker {}] received BufferFullEvent. Pausing production.",
                        self.id
                    );
                    self.is_production_stopped = true;
                }
                "BufferSpaceAvailableEvent" => {
                    log::info!(
                        "[Baker {}] received BufferSpaceAvailableEvent. Resuming production.",
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
                _ => {
                    log::warn!(
                        "[Baker {}] received unhandled event type: {}",
                        self.id,
                        event.event_type()
                    );
                }
            }
        }
        new_events
    }
}