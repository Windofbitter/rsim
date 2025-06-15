use crate::core::component::BaseComponent;
use crate::core::event::{Event, CycleAdvancedEvent};
use crate::core::event_manager::EventManager;
use crate::core::event_scheduler::EventScheduler;
use crate::core::types::ComponentId;
use crate::parallel::cross_thread_router::{CrossThreadEventRouter, CrossThreadEvent, ThreadEventReceiver};
use std::collections::HashMap;
use std::sync::{Arc, Barrier};
use log::debug;

/// Thread-local simulation engine that manages components assigned to a specific thread
pub struct ThreadLocalEngine {
    thread_id: usize,
    event_manager: EventManager,
    local_scheduler: EventScheduler,
    pending_remote_events: Vec<CrossThreadEvent>,
    current_cycle: u64,
    events_processed: u64,
    cross_thread_sent: u64,
    cross_thread_received: u64,
}

impl ThreadLocalEngine {
    /// Create a new thread-local engine
    pub fn new(thread_id: usize) -> Self {
        Self {
            thread_id,
            event_manager: EventManager::new(),
            local_scheduler: EventScheduler::new(),
            pending_remote_events: Vec::new(),
            current_cycle: 0,
            events_processed: 0,
            cross_thread_sent: 0,
            cross_thread_received: 0,
        }
    }
    
    /// Register a component with this thread-local engine
    pub fn register_component(&mut self, component: Box<dyn BaseComponent>) -> Result<(), String> {
        self.event_manager.register_component(component)
    }
    
    /// Schedule an event in the local scheduler
    pub fn schedule_event(&mut self, event: Box<dyn Event + Send + Sync>, delay_cycles: u64) {
        self.local_scheduler.schedule_event(event, delay_cycles);
    }
    
    /// Get thread ID
    pub fn thread_id(&self) -> usize {
        self.thread_id
    }
    
    /// Check if there are pending events
    pub fn has_pending_events(&self) -> bool {
        self.local_scheduler.has_events() || !self.pending_remote_events.is_empty()
    }
    
    /// Run simulation with synchronization
    pub fn run_with_synchronization(
        &mut self,
        barrier: Arc<Barrier>,
        router: Arc<CrossThreadEventRouter>,
        receiver: ThreadEventReceiver,
        max_cycles: Option<u64>,
    ) -> Result<u64, String> {
        
        loop {
            // Check cycle limit
            if let Some(max) = max_cycles {
                if self.current_cycle >= max {
                    return Ok(self.current_cycle);
                }
            }
            
            // Step 1: Process events for current time
            let has_local_events = self.process_current_time_events(&router)?;
            
            // Step 2: Synchronize with other threads
            barrier.wait();
            
            // Step 3: Collect cross-thread events
            self.collect_cross_thread_events(&receiver);
            
            // Step 4: Check if we should continue
            if !has_local_events && !self.has_pending_events() {
                // Try to advance to next event time or exit if no more events
                if let Some(next_delay) = self.local_scheduler.peek_next_delay() {
                    self.advance_time(next_delay);
                } else {
                    // No more local events, check if we're done
                    break;
                }
            }
            
            // Step 5: Final synchronization for this cycle
            barrier.wait();
        }
        
        Ok(self.current_cycle)
    }
    
    /// Process all events at the current time
    fn process_current_time_events(
        &mut self,
        router: &Arc<CrossThreadEventRouter>,
    ) -> Result<bool, String> {
        
        // First, schedule any pending remote events
        for remote_event in self.pending_remote_events.drain(..) {
            self.local_scheduler.schedule_event(remote_event.event, remote_event.delay);
            self.cross_thread_received += 1;
        }
        
        // Get all events for current time
        let current_time_events = self.local_scheduler.get_current_time_events();
        if current_time_events.is_empty() {
            return Ok(false);
        }
        
        debug!("[Thread {}] Processing {} events at cycle {}", 
               self.thread_id, current_time_events.len(), self.current_cycle);
        
        // Group events by component
        let mut events_by_component: HashMap<ComponentId, Vec<Box<dyn Event + Send + Sync>>> = HashMap::new();
        
        for event in current_time_events {
            let target_ids = self.event_manager.route_event(event.as_ref());
            
            for target_id in target_ids {
                // Check if this component is local or remote
                if self.event_manager.has_component(&target_id) {
                    // Local component
                    events_by_component
                        .entry(target_id)
                        .or_insert_with(Vec::new)
                        .push(event.clone_event());
                } else {
                    // Remote component - send through router
                    if let Err(e) = router.send_cross_thread_event(event.clone_event(), 0, &target_id) {
                        return Err(format!("Failed to send cross-thread event: {}", e));
                    }
                    self.cross_thread_sent += 1;
                }
            }
            
            self.events_processed += 1;
        }
        
        // Process events for each local component
        for (component_id, component_events) in events_by_component {
            if let Some(component) = self.event_manager.get_component_mut(&component_id) {
                let new_events = component.react_atomic(component_events);
                
                for (new_event, delay) in new_events {
                    // Check if any targets are remote
                    if let Some(target_ids) = new_event.target_ids() {
                        let mut has_remote_targets = false;
                        
                        for target_id in &target_ids {
                            if !self.event_manager.has_component(target_id) {
                                has_remote_targets = true;
                                if let Err(e) = router.send_cross_thread_event(new_event.clone_event(), delay, target_id) {
                                    return Err(format!("Failed to send cross-thread event: {}", e));
                                }
                                self.cross_thread_sent += 1;
                            }
                        }
                        
                        // Also schedule locally if there are local targets or no specific targets
                        if !has_remote_targets || target_ids.is_empty() {
                            self.local_scheduler.schedule_event(new_event, delay);
                        }
                    } else {
                        // Broadcast event - schedule locally
                        self.local_scheduler.schedule_event(new_event, delay);
                    }
                }
            }
        }
        
        Ok(true)
    }
    
    /// Collect cross-thread events from the receiver
    fn collect_cross_thread_events(&mut self, receiver: &ThreadEventReceiver) {
        let events = receiver.collect_available_events();
        self.pending_remote_events.extend(events);
    }
    
    /// Advance simulation time
    fn advance_time(&mut self, cycles: u64) {
        if cycles > 0 {
            let old_cycle = self.current_cycle;
            self.current_cycle += cycles;
            self.local_scheduler.advance_time(cycles);
            
            // Generate cycle advanced event
            let cycle_event = CycleAdvancedEvent::new(
                format!("thread_{}_engine", self.thread_id),
                None, // Broadcast to all local components
                old_cycle,
                self.current_cycle,
            );
            
            self.local_scheduler.schedule_event(Box::new(cycle_event), 0);
            
            debug!("[Thread {}] Advanced time from {} to {}", 
                   self.thread_id, old_cycle, self.current_cycle);
        }
    }
    
    /// Get statistics for this thread
    pub fn get_statistics(&self) -> ThreadStatistics {
        ThreadStatistics {
            thread_id: self.thread_id,
            events_processed: self.events_processed,
            components_count: self.event_manager.component_count(),
            cross_thread_events_sent: self.cross_thread_sent,
            cross_thread_events_received: self.cross_thread_received,
        }
    }
}

/// Statistics for thread execution
#[derive(Debug, Clone)]
pub struct ThreadStatistics {
    pub thread_id: usize,
    pub events_processed: u64,
    pub components_count: usize,
    pub cross_thread_events_sent: u64,
    pub cross_thread_events_received: u64,
}