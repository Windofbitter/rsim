use crate::analysis::{ComponentPartition, ThreadId};
use crate::core::event::Event;
use crate::core::types::ComponentId;
use std::sync::mpsc::{SyncSender, Receiver, sync_channel};

/// Event that needs to be delivered across threads
pub struct CrossThreadEvent {
    pub event: Box<dyn Event + Send + Sync>,
    pub delay: u64,
    pub target_thread: ThreadId,
}

impl Clone for CrossThreadEvent {
    fn clone(&self) -> Self {
        Self {
            event: self.event.clone_event(),
            delay: self.delay,
            target_thread: self.target_thread,
        }
    }
}

/// Router for handling cross-thread event communication
pub struct CrossThreadEventRouter {
    partition: ComponentPartition,
    senders: Vec<SyncSender<CrossThreadEvent>>,
    num_threads: usize,
    channel_capacity: usize,
}

/// Per-thread receiver for cross-thread events
pub struct ThreadEventReceiver {
    receiver: Receiver<CrossThreadEvent>,
    thread_id: ThreadId,
}

impl ThreadEventReceiver {
    /// Try to receive an event (non-blocking)
    pub fn try_receive_event(&self) -> Option<CrossThreadEvent> {
        self.receiver.try_recv().ok()
    }
    
    /// Collect all immediately available events
    pub fn collect_available_events(&self) -> Vec<CrossThreadEvent> {
        let mut events = Vec::new();
        
        while let Some(event) = self.try_receive_event() {
            events.push(event);
        }
        
        events
    }
    
    /// Get thread ID this receiver belongs to
    pub fn thread_id(&self) -> ThreadId {
        self.thread_id
    }
    
    /// Clear all pending events by consuming them
    pub fn clear_events(&self) {
        while self.receiver.try_recv().is_ok() {
            // Consume and discard events
        }
    }
}

impl CrossThreadEventRouter {
    /// Create a new cross-thread event router with default capacity
    pub fn new(partition: ComponentPartition, num_threads: usize) -> (Self, Vec<ThreadEventReceiver>) {
        Self::with_capacity(partition, num_threads, 4096)
    }
    
    /// Create a new cross-thread event router with specified capacity
    pub fn with_capacity(partition: ComponentPartition, num_threads: usize, capacity: usize) -> (Self, Vec<ThreadEventReceiver>) {
        let mut senders = Vec::with_capacity(num_threads);
        let mut receivers = Vec::with_capacity(num_threads);
        
        for thread_id in 0..num_threads {
            let (sender, receiver) = sync_channel(capacity);
            senders.push(sender);
            receivers.push(ThreadEventReceiver {
                receiver,
                thread_id,
            });
        }
        
        let router = Self {
            partition,
            senders,
            num_threads,
            channel_capacity: capacity,
        };
        
        (router, receivers)
    }
    
    /// Route an event to determine which threads should receive it
    pub fn route_event(&self, event: &dyn Event) -> Vec<ThreadId> {
        let mut target_threads = Vec::new();
        
        if let Some(target_ids) = event.target_ids() {
            // Specific targets - route to their threads
            for target_id in target_ids {
                if let Some(thread_id) = self.partition.get_thread_assignment(&target_id) {
                    if !target_threads.contains(&thread_id) {
                        target_threads.push(thread_id);
                    }
                }
            }
        } else {
            // Broadcast event - send to all threads
            for thread_id in 0..self.num_threads {
                target_threads.push(thread_id);
            }
        }
        
        target_threads
    }
    
    /// Send a cross-thread event to a specific component
    pub fn send_cross_thread_event(
        &self,
        event: Box<dyn Event + Send + Sync>,
        delay: u64,
        target_component: &ComponentId,
    ) -> Result<(), String> {
        if let Some(target_thread) = self.partition.get_thread_assignment(target_component) {
            self.send_to_thread(event, delay, target_thread)
        } else {
            Err(format!("Component {} not found in partition", target_component))
        }
    }
    
    /// Send an event directly to a specific thread
    pub fn send_to_thread(
        &self,
        event: Box<dyn Event + Send + Sync>,
        delay: u64,
        target_thread: ThreadId,
    ) -> Result<(), String> {
        if target_thread >= self.num_threads {
            return Err(format!("Invalid thread ID: {}", target_thread));
        }
        
        let cross_thread_event = CrossThreadEvent {
            event,
            delay,
            target_thread,
        };
        
        match self.senders[target_thread].send(cross_thread_event) {
            Ok(()) => Ok(()),
            Err(_) => Err(format!("Channel full or disconnected for thread {}", target_thread)),
        }
    }
    
    
    
    /// Get channel capacity
    pub fn channel_capacity(&self) -> usize {
        self.channel_capacity
    }
    
    /// Get number of threads
    pub fn num_threads(&self) -> usize {
        self.num_threads
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::CycleAdvancedEvent;
    
    #[test]
    fn test_cross_thread_router_creation() {
        let partition = ComponentPartition::new(4);
        let (router, receivers) = CrossThreadEventRouter::new(partition, 4);
        
        assert_eq!(router.num_threads(), 4);
        assert_eq!(receivers.len(), 4);
        assert_eq!(router.channel_capacity(), 4096);
    }
    
    #[test]
    fn test_send_and_collect_events() {
        let mut partition = ComponentPartition::new(2);
        partition.assign_component("comp1".to_string(), 0).unwrap();
        partition.assign_component("comp2".to_string(), 1).unwrap();
        
        let (router, receivers) = CrossThreadEventRouter::new(partition, 2);
        
        // Create a test event
        let event = Box::new(CycleAdvancedEvent::new(
            "test".to_string(),
            Some(vec!["comp2".to_string()]),
            0,
            1,
        ));
        
        // Send to thread 1
        router.send_cross_thread_event(event.clone_event(), 5, &"comp2".to_string()).unwrap();
        
        // Collect from thread 1
        let events = receivers[1].collect_available_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].delay, 5);
        assert_eq!(events[0].target_thread, 1);
        
        // Channel should be empty now
        let events = receivers[1].collect_available_events();
        assert_eq!(events.len(), 0);
    }
    
    #[test]
    fn test_broadcast_routing() {
        let partition = ComponentPartition::new(3);
        let (router, _receivers) = CrossThreadEventRouter::new(partition, 3);
        
        // Create a broadcast event (no target_ids)
        let event = CycleAdvancedEvent::new(
            "test".to_string(),
            None,
            0,
            1,
        );
        
        let target_threads = router.route_event(&event);
        assert_eq!(target_threads.len(), 3);
        assert!(target_threads.contains(&0));
        assert!(target_threads.contains(&1));
        assert!(target_threads.contains(&2));
    }
    
    #[test]
    fn test_bounded_capacity() {
        let partition = ComponentPartition::new(1);
        let (router, receivers) = CrossThreadEventRouter::with_capacity(partition, 1, 2); // Small capacity
        
        // Create test events
        let event1 = Box::new(CycleAdvancedEvent::new(
            "test1".to_string(),
            Some(vec!["comp1".to_string()]),
            0,
            1,
        ));
        let event2 = Box::new(CycleAdvancedEvent::new(
            "test2".to_string(),
            Some(vec!["comp1".to_string()]),
            0,
            2,
        ));
        let event3 = Box::new(CycleAdvancedEvent::new(
            "test3".to_string(),
            Some(vec!["comp1".to_string()]),
            0,
            3,
        ));
        
        // Send events up to capacity
        assert!(router.send_to_thread(event1, 0, 0).is_ok());
        assert!(router.send_to_thread(event2, 0, 0).is_ok());
        
        // This should fail due to capacity limit
        assert!(router.send_to_thread(event3, 0, 0).is_err());
        
        // After consuming one event, we should be able to send again
        let consumed = receivers[0].try_receive_event();
        assert!(consumed.is_some());
        
        let event4 = Box::new(CycleAdvancedEvent::new(
            "test4".to_string(),
            Some(vec!["comp1".to_string()]),
            0,
            4,
        ));
        assert!(router.send_to_thread(event4, 0, 0).is_ok());
    }
}