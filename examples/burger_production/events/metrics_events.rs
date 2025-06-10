use rsim::core::event::{Event, EventId};
use rsim::core::types::{ComponentId, ComponentValue};
use std::collections::HashMap;

pub const METRICS_REPORT_EVENT: &str = "MetricsReport";

/// Event for triggering periodic metrics reporting
#[derive(Debug, Clone)]
pub struct MetricsReportEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub report_time: u64,
}

impl Event for MetricsReportEvent {
    fn id(&self) -> &EventId {
        &self.id
    }

    fn event_type(&self) -> &str {
        METRICS_REPORT_EVENT
    }

    fn source_id(&self) -> &ComponentId {
        &self.source_id
    }

    fn target_ids(&self) -> Option<Vec<ComponentId>> {
        None // Broadcast event
    }

    fn data(&self) -> HashMap<String, ComponentValue> {
        let mut data = HashMap::new();
        data.insert("report_time".to_string(), ComponentValue::Int(self.report_time as i64));
        data
    }

    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}