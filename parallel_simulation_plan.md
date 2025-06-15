# Parallel Simulation Implementation Plan

## Overview

This document outlines the plan for implementing parallel execution support in the rsim event-driven simulator. The goal is to leverage the existing profiling and partitioning infrastructure to execute simulations across multiple threads while maintaining deterministic behavior.

## Current State

### Completed Components
- **Profiling Infrastructure**: `ProfilingCollector` tracks event flows and component interactions
- **Dependency Graph**: Captures component relationships and communication patterns
- **Graph Partitioner**: `GreedyPartitioner` implements load-balanced partitioning
- **Component Partition**: Maps components to threads with quality metrics
- **Profiled Simulation Run**: Orchestrates profiling → partitioning pipeline

### Core Simulation Components
- `SimulationEngine`: Single-threaded event processing
- `EventScheduler`: Priority queue-based event scheduling
- `EventManager`: Component registry and event routing
- `BaseComponent`: Component trait with atomic event processing

## Design Principles

1. **Minimal Core Changes**: Keep `SimulationEngine` focused on execution logic
2. **Deterministic Execution**: Maintain reproducibility across parallel runs
3. **Clean Separation**: Separate parallelization concerns from core simulation
4. **Performance Focus**: Minimize cross-thread communication overhead
5. **Flexible Architecture**: Support different parallelization strategies

## Proposed Architecture

### High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                   ParallelSimulationBuilder                 │
│  (High-level API for configuring and running parallel sims) │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                   ParallelSimulationOrchestrator            │
│  (Coordinates profiling, partitioning, and parallel exec)   │
└─────────────────────────┬───────────────────────────────────┘
                          │
        ┌─────────────────┴─────────────────┐
        │                                   │
┌───────▼────────┐              ┌──────────▼──────────┐
│ ProfiledSimRun │              │ ParallelSimEngine   │
│  (Profiling &  │              │ (Multi-threaded     │
│  Partitioning) │              │  execution engine)  │
└────────────────┘              └─────────────────────┘
```

### Thread Architecture

```
Thread 0          Thread 1          Thread 2          Thread N
   │                 │                 │                 │
   ▼                 ▼                 ▼                 ▼
┌──────┐         ┌──────┐         ┌──────┐         ┌──────┐
│Local │         │Local │         │Local │         │Local │
│Event │         │Event │         │Event │         │Event │
│Queue │         │Event │         │Queue │         │Queue │
└──┬───┘         └──┬───┘         └──┬───┘         └──┬───┘
   │                │                │                │
   ▼                ▼                ▼                ▼
┌──────┐         ┌──────┐         ┌──────┐         ┌──────┐
│Comp  │         │Comp  │         │Comp  │         │Comp  │
│Set 0 │         │Set 1 │         │Set 2 │         │Set N │
└──────┘         └──────┘         └──────┘         └──────┘
   │                │                │                │
   └────────────────┴────────────────┴────────────────┘
                           │
                    ┌──────▼──────┐
                    │Cross-Thread │
                    │Event Router │
                    └─────────────┘
```

## Implementation Phases

### Phase 1: Core Parallel Execution Engine

#### 1.1 Create `ParallelSimulationEngine`
```rust
// src/parallel/parallel_simulation_engine.rs
pub struct ParallelSimulationEngine {
    thread_engines: Vec<ThreadLocalEngine>,
    cross_thread_router: CrossThreadEventRouter,
    partition: ComponentPartition,
    synchronization: SynchronizationBarrier,
}
```

#### 1.2 Implement `ThreadLocalEngine`
```rust
// src/parallel/thread_local_engine.rs
pub struct ThreadLocalEngine {
    thread_id: ThreadId,
    components: HashMap<ComponentId, Box<dyn BaseComponent>>,
    local_scheduler: EventScheduler,
    pending_remote_events: Vec<(Box<dyn Event>, u64, ThreadId)>,
}
```

#### 1.3 Create `CrossThreadEventRouter`
```rust
// src/parallel/cross_thread_router.rs
pub struct CrossThreadEventRouter {
    partition: Arc<ComponentPartition>,
    thread_queues: Vec<Arc<Mutex<Vec<CrossThreadEvent>>>>,
}
```

### Phase 2: Synchronization and Determinism

#### 2.1 Implement Time Synchronization
- Global time barrier synchronization
- Consistent event ordering across threads
- Deterministic tie-breaking for simultaneous events

#### 2.2 Create Event Serialization
```rust
// src/parallel/event_serialization.rs
pub trait SerializableEvent: Event {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Result<Box<dyn Event>, String>;
}
```

#### 2.3 Implement Deterministic Execution
- Stable component ID ordering
- Reproducible random number generation per thread
- Consistent cross-thread event delivery order

### Phase 3: High-Level Orchestration

#### 3.1 Create `ParallelSimulationOrchestrator`
```rust
// src/parallel/orchestrator.rs
pub struct ParallelSimulationOrchestrator {
    profiler: ProfiledSimulationRun,
    partitioner_name: String,
    num_threads: usize,
}

impl ParallelSimulationOrchestrator {
    pub fn run_complete_pipeline<F>(
        &self,
        component_factory: F,
        initial_events: Vec<(Box<dyn Event>, u64)>,
    ) -> Result<ParallelSimulationResult, String>
    where
        F: Fn() -> Vec<Box<dyn BaseComponent>> + Send + Sync + Clone,
    {
        // 1. Run profiling
        // 2. Partition components
        // 3. Execute parallel simulation
        // 4. Collect results
    }
}
```

#### 3.2 Implement `ParallelSimulationBuilder`
```rust
// src/parallel/builder.rs
pub struct ParallelSimulationBuilder {
    component_factory: Option<Box<dyn Fn() -> Vec<Box<dyn BaseComponent>>>>,
    initial_events: Vec<(Box<dyn Event>, u64)>,
    profiling_cycles: Option<u64>,
    partitioner_name: String,
    num_threads: usize,
}

impl ParallelSimulationBuilder {
    pub fn new() -> Self { ... }
    pub fn with_components<F>(mut self, factory: F) -> Self { ... }
    pub fn with_initial_events(mut self, events: Vec<(Box<dyn Event>, u64)>) -> Self { ... }
    pub fn profile_cycles(mut self, cycles: u64) -> Self { ... }
    pub fn partition_algorithm(mut self, name: &str) -> Self { ... }
    pub fn num_threads(mut self, threads: usize) -> Self { ... }
    pub fn run(self) -> Result<ParallelSimulationResult, String> { ... }
}
```

### Phase 4: Performance Optimizations

#### 4.1 Lock-Free Data Structures
- Implement lock-free cross-thread event queues
- Use atomic operations for statistics collection
- Minimize contention points

#### 4.2 Event Batching
- Batch cross-thread events to reduce synchronization
- Implement adaptive batching based on load
- Profile and optimize batch sizes

#### 4.3 Work Stealing
- Implement work-stealing for load balancing
- Dynamic component migration (optional)
- Adaptive repartitioning for long-running simulations

## Integration with Existing Code

### Minimal Changes to Core
1. Add `Send + Sync` bounds to `BaseComponent` trait
2. Make `Event` trait implement `Send + Sync`
3. Add event serialization support (for cross-thread events)
4. No changes to `SimulationEngine` core logic

### New Module Structure
```
src/
├── parallel/
│   ├── mod.rs
│   ├── parallel_simulation_engine.rs
│   ├── thread_local_engine.rs
│   ├── cross_thread_router.rs
│   ├── event_serialization.rs
│   ├── synchronization.rs
│   ├── orchestrator.rs
│   └── builder.rs
├── analysis/           # Existing
├── core/              # Existing
└── components/        # Existing
```

## Usage Example

```rust
use rsim::parallel::ParallelSimulationBuilder;

fn main() -> Result<(), String> {
    let result = ParallelSimulationBuilder::new()
        .with_components(|| create_burger_production_components())
        .with_initial_events(vec![
            (Box::new(StartEvent::new()), 0),
        ])
        .profile_cycles(1000)
        .partition_algorithm("greedy")
        .num_threads(4)
        .run()?;

    result.print_summary();
    println!("Speedup: {:.2}x", result.speedup());
    
    Ok(())
}
```

## Testing Strategy

### Unit Tests
- Thread-local engine event processing
- Cross-thread event routing correctness
- Synchronization barrier behavior
- Determinism verification

### Integration Tests
- Compare single-threaded vs multi-threaded results
- Verify deterministic execution across runs
- Test various partition configurations
- Benchmark performance improvements

### Performance Tests
- Measure synchronization overhead
- Profile cross-thread communication costs
- Analyze scalability with thread count
- Identify bottlenecks and optimization opportunities

## Metrics and Monitoring

### Performance Metrics
- **Simulation Time**: Wall-clock time for completion
- **Speedup**: Ratio vs single-threaded execution
- **Efficiency**: Speedup / number of threads
- **Load Balance**: Thread utilization variance
- **Communication Overhead**: Cross-thread event percentage

### Debugging Support
- Per-thread event logs
- Cross-thread event tracing
- Deadlock detection
- Performance profiling hooks

## Risk Mitigation

### Technical Risks
1. **Deadlocks**: Use careful lock ordering and timeouts
2. **Non-determinism**: Extensive testing and verification
3. **Performance Regression**: Continuous benchmarking
4. **Memory Overhead**: Monitor and optimize data structures

### Mitigation Strategies
- Incremental implementation with thorough testing
- Fallback to single-threaded mode
- Configurable synchronization strategies
- Performance regression tests in CI

## Future Extensions

1. **GPU Acceleration**: Offload compute-intensive components
2. **Distributed Simulation**: Multi-machine execution
3. **Dynamic Repartitioning**: Runtime load balancing
4. **Checkpointing**: Save/restore simulation state
5. **Visualization**: Real-time parallel execution monitoring

## Timeline Estimate

- **Phase 1**: 2-3 weeks (Core parallel engine)
- **Phase 2**: 1-2 weeks (Synchronization and determinism)
- **Phase 3**: 1 week (High-level orchestration)
- **Phase 4**: 2-3 weeks (Performance optimizations)
- **Testing & Documentation**: 1-2 weeks

**Total**: 7-11 weeks for complete implementation

## Success Criteria

1. **Correctness**: Parallel execution produces identical results to single-threaded
2. **Performance**: Achieve >2x speedup on 4 threads for typical workloads
3. **Scalability**: Linear speedup up to 8 threads for well-partitioned graphs
4. **Usability**: Simple API requiring minimal code changes
5. **Reliability**: No deadlocks or race conditions in production use