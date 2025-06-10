use rsim::core::component::BaseComponent;
use rsim::core::event::Event;
use rsim::core::types::{ComponentId, ComponentValue};
use uuid::Uuid;
use log::{info, debug};

use crate::events::{
    GENERATE_ORDER_EVENT, ITEM_ADDED_EVENT,
    MetricsReportEvent, METRICS_REPORT_EVENT
};

#[derive(Debug, Clone)]
pub struct ThroughputMetrics {
    // Counters for current window
    pub meat_produced: u32,
    pub bread_produced: u32, 
    pub burgers_produced: u32,
    pub orders_generated: u32,
    pub items_buffered: u32,
    
    // Time tracking
    pub window_start: u64,
    pub window_size: u64,
    pub current_time: u64,
    
    // Historical rates (items per time unit)
    pub meat_rate: f64,
    pub bread_rate: f64,
    pub burger_rate: f64,
    pub order_rate: f64,
    pub buffering_rate: f64,
    
    // Cumulative totals
    pub total_meat_produced: u32,
    pub total_bread_produced: u32,
    pub total_burgers_produced: u32,
    pub total_orders_generated: u32,
}

impl ThroughputMetrics {
    pub fn new(window_size: u64) -> Self {
        Self {
            meat_produced: 0,
            bread_produced: 0,
            burgers_produced: 0,
            orders_generated: 0,
            items_buffered: 0,
            window_start: 0,
            window_size,
            current_time: 0,
            meat_rate: 0.0,
            bread_rate: 0.0,
            burger_rate: 0.0,
            order_rate: 0.0,
            buffering_rate: 0.0,
            total_meat_produced: 0,
            total_bread_produced: 0,
            total_burgers_produced: 0,
            total_orders_generated: 0,
        }
    }

    pub fn calculate_rates(&mut self) {
        let window_duration = self.current_time - self.window_start;
        if window_duration > 0 {
            self.meat_rate = self.meat_produced as f64 / window_duration as f64;
            self.bread_rate = self.bread_produced as f64 / window_duration as f64;
            self.burger_rate = self.burgers_produced as f64 / window_duration as f64;
            self.order_rate = self.orders_generated as f64 / window_duration as f64;
            self.buffering_rate = self.items_buffered as f64 / window_duration as f64;
        }
    }

    pub fn reset_window(&mut self, current_time: u64) {
        // Update cumulative totals
        self.total_meat_produced += self.meat_produced;
        self.total_bread_produced += self.bread_produced;
        self.total_burgers_produced += self.burgers_produced;
        self.total_orders_generated += self.orders_generated;
        
        // Reset window counters
        self.meat_produced = 0;
        self.bread_produced = 0;
        self.burgers_produced = 0;
        self.orders_generated = 0;
        self.items_buffered = 0;
        self.window_start = current_time;
        self.current_time = current_time;
    }
}

/// Component that collects throughput metrics from production events
#[derive(Debug)]
pub struct MetricsCollector {
    pub component_id: ComponentId,
    pub metrics: ThroughputMetrics,
    pub report_interval: u64,
    pub last_report_time: u64,
}

impl MetricsCollector {
    pub fn new(component_id: ComponentId, window_size: u64, report_interval: u64) -> Self {
        Self {
            component_id,
            metrics: ThroughputMetrics::new(window_size),
            report_interval,
            last_report_time: 0,
        }
    }

    fn create_metrics_report_event(&self, current_time: u64) -> Box<dyn Event> {
        Box::new(MetricsReportEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            report_time: current_time,
        })
    }


    fn handle_order_generated(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        self.metrics.orders_generated += 1;
        debug!("[MetricsCollector:{}] Recorded order generation (window total: {})", 
               self.component_id, self.metrics.orders_generated);
        Vec::new()
    }

    fn handle_item_added(&mut self, event: &dyn Event) -> Vec<(Box<dyn Event>, u64)> {
        let data = event.data();
        if let Some(ComponentValue::String(item_type)) = data.get("item_type") {
            match item_type.as_str() {
                "meat" => {
                    self.metrics.meat_produced += 1;
                    debug!("[MetricsCollector:{}] Recorded meat production (window total: {})", 
                           self.component_id, self.metrics.meat_produced);
                }
                "bread" => {
                    self.metrics.bread_produced += 1;
                    debug!("[MetricsCollector:{}] Recorded bread production (window total: {})", 
                           self.component_id, self.metrics.bread_produced);
                }
                "burger" => {
                    self.metrics.burgers_produced += 1;
                    debug!("[MetricsCollector:{}] Recorded burger production (window total: {})", 
                           self.component_id, self.metrics.burgers_produced);
                }
                _ => {
                    // Unknown item type, count as general buffering
                    self.metrics.items_buffered += 1;
                    debug!("[MetricsCollector:{}] Recorded unknown item buffered: {} (window total: {})", 
                           self.component_id, item_type, self.metrics.items_buffered);
                }
            }
        } else {
            // No item_type data, count as general buffering
            self.metrics.items_buffered += 1;
            debug!("[MetricsCollector:{}] Recorded item buffered (no type) (window total: {})", 
                   self.component_id, self.metrics.items_buffered);
        }
        Vec::new()
    }

    fn handle_metrics_report(&mut self, current_time: u64) -> Vec<(Box<dyn Event>, u64)> {
        self.metrics.current_time = current_time;
        self.metrics.calculate_rates();
        
        info!("[MetricsCollector:{}] === THROUGHPUT REPORT (Time: {}) ===", 
              self.component_id, current_time);
        info!("[MetricsCollector:{}] Window: {} cycles (from {} to {})", 
              self.component_id, current_time - self.metrics.window_start, 
              self.metrics.window_start, current_time);
        info!("[MetricsCollector:{}] Production Rates (items/cycle):", self.component_id);
        info!("[MetricsCollector:{}]   Meat:    {:.3} ({} items)", 
              self.component_id, self.metrics.meat_rate, self.metrics.meat_produced);
        info!("[MetricsCollector:{}]   Bread:   {:.3} ({} items)", 
              self.component_id, self.metrics.bread_rate, self.metrics.bread_produced);
        info!("[MetricsCollector:{}]   Burgers: {:.3} ({} items)", 
              self.component_id, self.metrics.burger_rate, self.metrics.burgers_produced);
        info!("[MetricsCollector:{}]   Orders:  {:.3} ({} orders)", 
              self.component_id, self.metrics.order_rate, self.metrics.orders_generated);
        info!("[MetricsCollector:{}]   Buffering: {:.3} ({} items)", 
              self.component_id, self.metrics.buffering_rate, self.metrics.items_buffered);
        info!("[MetricsCollector:{}] Cumulative Totals:", self.component_id);
        info!("[MetricsCollector:{}]   Total Meat: {}", 
              self.component_id, self.metrics.total_meat_produced + self.metrics.meat_produced);
        info!("[MetricsCollector:{}]   Total Bread: {}", 
              self.component_id, self.metrics.total_bread_produced + self.metrics.bread_produced);
        info!("[MetricsCollector:{}]   Total Burgers: {}", 
              self.component_id, self.metrics.total_burgers_produced + self.metrics.burgers_produced);
        info!("[MetricsCollector:{}]   Total Orders: {}", 
              self.component_id, self.metrics.total_orders_generated + self.metrics.orders_generated);
        info!("[MetricsCollector:{}] ================================", self.component_id);

        // Reset window and schedule next report
        self.metrics.reset_window(current_time);
        self.last_report_time = current_time;
        
        let next_report_event = self.create_metrics_report_event(current_time);
        vec![(next_report_event, self.report_interval)]
    }

    pub fn schedule_initial_report(&self, _initial_delay: u64) -> Box<dyn Event> {
        self.create_metrics_report_event(0)
    }

    pub fn get_current_metrics(&self) -> &ThroughputMetrics {
        &self.metrics
    }
}

impl BaseComponent for MetricsCollector {
    fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &[
            GENERATE_ORDER_EVENT,
            ITEM_ADDED_EVENT,
            METRICS_REPORT_EVENT,
        ]
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        for event in events {
            match event.event_type() {
                GENERATE_ORDER_EVENT => {
                    let mut new_events = self.handle_order_generated();
                    output_events.append(&mut new_events);
                }
                ITEM_ADDED_EVENT => {
                    let mut new_events = self.handle_item_added(event.as_ref());
                    output_events.append(&mut new_events);
                }
                METRICS_REPORT_EVENT => {
                    // Only handle our own report events
                    if event.source_id() == &self.component_id {
                        if let Some(ComponentValue::Int(report_time)) = event.data().get("report_time") {
                            let mut new_events = self.handle_metrics_report(*report_time as u64);
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