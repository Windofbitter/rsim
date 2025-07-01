//! Metrics collection component for burger production simulation analysis.

use rsim::core::{
    component::BaseComponent,
    event::Event,
    types::{ComponentId, ComponentValue},
};
use std::collections::HashMap;

/// Tracks timing for individual orders
#[derive(Debug, Clone)]
struct OrderTiming {
    placement_cycle: u64,
}

/// Summary metrics for the simulation
#[derive(Debug, Clone)]
pub struct SimulationMetrics {
    pub total_orders_generated: u32,
    pub total_orders_fulfilled: u32,
    pub orders_fulfilled_per_cycle: f64,
    pub average_fulfillment_time: f64,
    pub max_fulfillment_time: u64,
    pub min_fulfillment_time: u64,
    pub simulation_duration: u64,
}

impl Default for SimulationMetrics {
    fn default() -> Self {
        Self {
            total_orders_generated: 0,
            total_orders_fulfilled: 0,
            orders_fulfilled_per_cycle: 0.0,
            average_fulfillment_time: 0.0,
            max_fulfillment_time: 0,
            min_fulfillment_time: u64::MAX,
            simulation_duration: 0,
        }
    }
}

/// Metrics collector component that tracks order fulfillment performance
pub struct MetricsCollector {
    id: ComponentId,
    subscriptions: Vec<&'static str>,

    // Order tracking
    pending_orders: HashMap<String, OrderTiming>,
    fulfilled_orders: Vec<(String, u64, u64)>, // (order_id, placement_cycle, fulfillment_cycle)

    // Cycle tracking
    current_cycle: u64,
    orders_fulfilled_this_cycle: u32,
    fulfillment_times: Vec<u64>,
    last_activity_cycle: u64,

    // Statistics
    total_orders_generated: u32,
    total_orders_fulfilled: u32,
}

impl MetricsCollector {
    /// Creates a new metrics collector component
    pub fn new(id: ComponentId) -> Self {
        log::info!("🔧 Creating MetricsCollector with ID: {}", id);
        Self {
            id,
            subscriptions: vec![
                "GenerateOrderEvent",
                "PlaceOrderEvent",
                "OrderFulfilledEvent",
                "ItemDispatchedEvent", // To track when burgers are delivered to client
                "CycleAdvancedEvent",  // To track simulation cycle progression
            ],
            pending_orders: HashMap::new(),
            fulfilled_orders: Vec::new(),
            current_cycle: 0,
            orders_fulfilled_this_cycle: 0,
            fulfillment_times: Vec::new(),
            last_activity_cycle: 0,
            total_orders_generated: 0,
            total_orders_fulfilled: 0,
        }
    }

    /// Handle order generation event
    fn handle_generate_order(&mut self, _event: &dyn Event) {
        self.total_orders_generated += 1;
        self.last_activity_cycle = self.current_cycle;

        // In both modes, order generation represents the start of an order
        // Generate a synthetic order ID for tracking
        let order_id = format!(
            "generated_order_{}_cycle_{}",
            self.total_orders_generated, self.current_cycle
        );
        let timing = OrderTiming {
            placement_cycle: self.current_cycle,
        };

        self.pending_orders.insert(order_id.clone(), timing);

        log::debug!(
            "[MetricsCollector {}] Order {} generated at cycle {}. Total generated: {}",
            self.id,
            order_id,
            self.current_cycle,
            self.total_orders_generated
        );
    }

    /// Handle order placement event
    fn handle_place_order(&mut self, event: &dyn Event) {
        let data = event.data();
        let order_id = data
            .get("order_id")
            .and_then(|v| {
                if let ComponentValue::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| format!("unknown_order_{}", self.current_cycle));

        // Record when this order was placed
        let timing = OrderTiming {
            placement_cycle: self.current_cycle,
        };

        self.pending_orders.insert(order_id.clone(), timing);
        self.last_activity_cycle = self.current_cycle;

        log::debug!(
            "[MetricsCollector {}] Order {} placed at cycle {}",
            self.id,
            order_id,
            self.current_cycle
        );
    }

    /// Handle cycle advancement event  
    fn handle_cycle_advanced(&mut self, event: &dyn Event) {
        let data = event.data();
        let new_cycle = data
            .get("new_cycle")
            .and_then(|v| {
                if let ComponentValue::Int(cycle) = v {
                    Some(*cycle as u64)
                } else {
                    None
                }
            })
            .unwrap_or(self.current_cycle);

        if new_cycle > self.current_cycle {
            // Reset per-cycle counters when cycle advances
            self.orders_fulfilled_this_cycle = 0;
        }

        self.current_cycle = new_cycle;
        log::debug!(
            "[MetricsCollector {}] Cycle updated to {} (via event)",
            self.id,
            self.current_cycle
        );
    }

    /// Handle order fulfillment event
    fn handle_order_fulfilled(&mut self, event: &dyn Event) {
        let data = event.data();
        let order_id = data
            .get("order_id")
            .and_then(|v| {
                if let ComponentValue::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| format!("unknown_order_{}", self.current_cycle));

        // Find the pending order and calculate fulfillment time
        // If exact match not found, use the oldest pending order (FIFO assumption)
        let timing = if let Some(timing) = self.pending_orders.remove(&order_id) {
            timing
        } else if !self.pending_orders.is_empty() {
            // Use oldest pending order (FIFO assumption for order fulfillment)
            // Find order with minimum placement_cycle (truly oldest)
            let oldest_key = self
                .pending_orders
                .iter()
                .min_by_key(|(_, timing)| timing.placement_cycle)
                .map(|(key, _)| key.clone());
            if let Some(key) = oldest_key {
                self.pending_orders.remove(&key).unwrap()
            } else {
                log::warn!(
                    "[MetricsCollector {}] Order {} fulfilled but no pending orders found",
                    self.id,
                    order_id
                );
                return;
            }
        } else {
            log::warn!(
                "[MetricsCollector {}] Order {} fulfilled but no pending orders available",
                self.id,
                order_id
            );
            return;
        };

        let fulfillment_time = self.current_cycle - timing.placement_cycle;

        // Record the fulfilled order
        self.fulfilled_orders
            .push((order_id.clone(), timing.placement_cycle, self.current_cycle));

        // Update statistics
        self.total_orders_fulfilled += 1;
        self.orders_fulfilled_this_cycle += 1;
        self.fulfillment_times.push(fulfillment_time);
        self.last_activity_cycle = self.current_cycle;

        log::info!(
            "[MetricsCollector {}] Order {} fulfilled at cycle {} (took {} cycles). Total fulfilled: {}",
            self.id,
            order_id,
            self.current_cycle,
            fulfillment_time,
            self.total_orders_fulfilled
        );

        // Print periodic summary every 5 fulfilled orders
        if self.total_orders_fulfilled % 5 == 0 {
            self.print_periodic_summary();
        }
    }

    /// Calculate final metrics for the simulation
    pub fn calculate_final_metrics(&self) -> SimulationMetrics {
        let average_fulfillment_time = if !self.fulfillment_times.is_empty() {
            self.fulfillment_times.iter().sum::<u64>() as f64 / self.fulfillment_times.len() as f64
        } else {
            0.0
        };

        let orders_fulfilled_per_cycle = if self.current_cycle > 0 {
            self.total_orders_fulfilled as f64 / self.current_cycle as f64
        } else {
            0.0
        };

        let max_fulfillment_time = self.fulfillment_times.iter().max().copied().unwrap_or(0);
        let min_fulfillment_time = if !self.fulfillment_times.is_empty() {
            self.fulfillment_times
                .iter()
                .min()
                .copied()
                .unwrap_or(u64::MAX)
        } else {
            u64::MAX
        };

        SimulationMetrics {
            total_orders_generated: self.total_orders_generated,
            total_orders_fulfilled: self.total_orders_fulfilled,
            orders_fulfilled_per_cycle,
            average_fulfillment_time,
            max_fulfillment_time,
            min_fulfillment_time,
            simulation_duration: self.current_cycle,
        }
    }

    /// Print periodic metrics summary (less detailed)
    pub fn print_periodic_summary(&self) {
        let metrics = self.calculate_final_metrics();

        log::info!("📊 METRICS UPDATE [Cycle {}] - Orders: {}/{} fulfilled, Avg time: {:.1} cycles, Rate: {:.3}/cycle",
            self.current_cycle,
            metrics.total_orders_fulfilled,
            metrics.total_orders_generated,
            metrics.average_fulfillment_time,
            metrics.orders_fulfilled_per_cycle
        );
    }

    /// Print current metrics summary
    pub fn print_metrics_summary(&self) {
        let metrics = self.calculate_final_metrics();

        log::info!("═══════════════════════════════════════");
        log::info!("        METRICS SUMMARY");
        log::info!("═══════════════════════════════════════");
        log::info!("Total Orders Generated: {}", metrics.total_orders_generated);
        log::info!("Total Orders Fulfilled: {}", metrics.total_orders_fulfilled);
        log::info!(
            "Orders Fulfilled Per Cycle: {:.3}",
            metrics.orders_fulfilled_per_cycle
        );
        log::info!(
            "Average Fulfillment Time: {:.2} cycles",
            metrics.average_fulfillment_time
        );
        log::info!(
            "Min Fulfillment Time: {}",
            if metrics.min_fulfillment_time == u64::MAX {
                "N/A".to_string()
            } else {
                format!("{} cycles", metrics.min_fulfillment_time)
            }
        );
        log::info!(
            "Max Fulfillment Time: {} cycles",
            metrics.max_fulfillment_time
        );
        log::info!(
            "Simulation Duration: {} cycles",
            metrics.simulation_duration
        );
        log::info!("═══════════════════════════════════════");
    }
}

impl Drop for MetricsCollector {
    fn drop(&mut self) {
        log::info!("🔚 Simulation complete - printing final metrics summary");
        self.print_metrics_summary();
    }
}

impl BaseComponent for MetricsCollector {
    fn component_id(&self) -> &ComponentId {
        &self.id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &self.subscriptions
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        // Cycle tracking is now handled via Observer pattern
        if !events.is_empty() {
            log::debug!(
                "[MetricsCollector {}] Processing {} events at cycle {}",
                self.id,
                events.len(),
                self.current_cycle
            );
        }

        for event in events {
            log::debug!(
                "[MetricsCollector {}] Processing event type: {}",
                self.id,
                event.event_type()
            );
            match event.event_type() {
                "GenerateOrderEvent" => {
                    log::info!("[MetricsCollector {}] Received GenerateOrderEvent", self.id);
                    self.handle_generate_order(event.as_ref());
                }
                "PlaceOrderEvent" => {
                    log::info!("[MetricsCollector {}] Received PlaceOrderEvent", self.id);
                    self.handle_place_order(event.as_ref());
                }
                "OrderFulfilledEvent" => {
                    log::info!(
                        "[MetricsCollector {}] Received OrderFulfilledEvent",
                        self.id
                    );
                    self.handle_order_fulfilled(event.as_ref());
                }
                "ItemDispatchedEvent" => {
                    log::debug!(
                        "[MetricsCollector {}] Received ItemDispatchedEvent (ignoring)",
                        self.id
                    );
                    // Just track that we received it but don't process
                }
                "CycleAdvancedEvent" => {
                    log::debug!("[MetricsCollector {}] Received CycleAdvancedEvent", self.id);
                    self.handle_cycle_advanced(event.as_ref());
                }
                _ => {
                    // Ignore other event types
                    log::debug!(
                        "[MetricsCollector {}] Ignoring event type: {}",
                        self.id,
                        event.event_type()
                    );
                }
            }
        }

        // Metrics collector doesn't generate new events
        Vec::new()
    }
}
