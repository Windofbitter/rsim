use super::event::Event;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug)]
pub struct ScheduledEvent {
    pub delay_cycles: u64,
    pub sequence_num: u64,
    pub event: Box<dyn Event>,
}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.delay_cycles == other.delay_cycles && self.sequence_num == other.sequence_num
    }
}

impl Eq for ScheduledEvent {}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (BinaryHeap is max-heap by default)
        other
            .delay_cycles
            .cmp(&self.delay_cycles)
            .then_with(|| other.sequence_num.cmp(&self.sequence_num))
    }
}

pub struct EventScheduler {
    event_queue: BinaryHeap<ScheduledEvent>,
    sequence_counter: u64,
}

impl EventScheduler {
    /// Create a new EventScheduler
    pub fn new() -> Self {
        Self {
            event_queue: BinaryHeap::new(),
            sequence_counter: 0,
        }
    }

    /// Schedule an event to execute after the specified delay
    pub fn schedule_event(&mut self, event: Box<dyn Event>, delay_cycles: u64) {
        let scheduled_event = ScheduledEvent {
            delay_cycles,
            sequence_num: self.sequence_counter,
            event,
        };

        self.event_queue.push(scheduled_event);
        self.sequence_counter += 1;
    }

    /// Get all events scheduled for the next time step (minimum delay)
    pub fn get_next_time_events(&mut self) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        if let Some(next_delay) = self.peek_next_delay() {
            while let Some(scheduled_event) = self.event_queue.peek() {
                if scheduled_event.delay_cycles == next_delay {
                    let scheduled_event = self.event_queue.pop().unwrap();
                    events.push(scheduled_event.event);
                } else {
                    break;
                }
            }
        }

        events
    }

    /// Check if there are any events remaining in the queue
    pub fn has_events(&self) -> bool {
        !self.event_queue.is_empty()
    }

    /// Get the next event delay without removing events
    pub fn peek_next_delay(&self) -> Option<u64> {
        self.event_queue.peek().map(|event| event.delay_cycles)
    }

    /// Advance time by reducing all event delays by the specified amount
    pub fn advance_time(&mut self, cycles: u64) {
        let mut temp_events = Vec::new();

        while let Some(mut scheduled_event) = self.event_queue.pop() {
            scheduled_event.delay_cycles = scheduled_event.delay_cycles.saturating_sub(cycles);
            temp_events.push(scheduled_event);
        }

        for event in temp_events {
            self.event_queue.push(event);
        }
    }
}
