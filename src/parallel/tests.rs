#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::ComponentPartition;
    use crate::core::event::CycleAdvancedEvent;
    use crate::parallel::{ParallelSimulationEngine, CrossThreadEventRouter};
    
    #[test]
    fn test_parallel_engine_creation() {
        let partition = ComponentPartition::new(2);
        let engine = ParallelSimulationEngine::new(partition, Some(100));
        
        assert_eq!(engine.current_cycle(), 0);
        assert_eq!(engine.num_threads, 2);
    }
    
    #[test]
    fn test_cross_thread_router_basic() {
        let partition = ComponentPartition::new(2);
        let router = CrossThreadEventRouter::new(partition, 2);
        
        assert_eq!(router.total_pending_events(), 0);
        assert!(!router.has_pending_events());
    }
    
    #[test]
    fn test_thread_local_engine_creation() {
        let engine = ThreadLocalEngine::new(0);
        
        assert_eq!(engine.thread_id(), 0);
        assert!(!engine.has_pending_events());
    }
}