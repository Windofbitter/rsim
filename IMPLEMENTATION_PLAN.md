# R-Sim Parallel Engine: Implementation Plan

This document outlines a phased implementation plan for evolving `rsim` from a single-threaded simulation engine into a parallel one, based on the `PARALLEL_SIMULATION_DESIGN.md` document.

## Phase 0: Core Data Structures & Project Setup

This phase lays the groundwork by defining the necessary data structures for inter-thread communication and adding dependencies.

1.  **Add Dependencies**:
    *   Add a graph partitioning library to `Cargo.toml`. `metis-rs` is a good candidate.
    *   Add a high-performance channel library, such as `crossbeam-channel`.

2.  **Define Core Communication Types**:
    *   Create a new module, e.g., `rsim::parallel`.
    *   Define the messages that will be passed between threads. This will be the lifeblood of the synchronization system.

    ```rust
    // In a new file, e.g., src/parallel/types.rs

    use crate::{Event, ComponentId, Time};

    /// A message sent from one thread's scheduler to another.
    pub enum CrossThreadMessage {
        /// An event targeted at a component on the receiving thread.
        Event(ExternalEvent),
        /// A message containing only an updated lookahead guarantee, used to prevent deadlock.
        Null(NullMessage),
    }

    /// An event that crosses a thread boundary.
    pub struct ExternalEvent {
        /// The component on the destination thread that this event is for.
        pub target_component: ComponentId,
        /// The actual event payload.
        pub event: Event,
        /// The new lookahead guarantee from the sender's thread.
        pub lookahead_guarantee: Time,
    }

    /// A message to communicate an updated lookahead guarantee without an associated event.
    pub struct NullMessage {
        /// The new lookahead guarantee from the sender's thread.
        pub lookahead_guarantee: Time,
    }
    ```

## Phase 1: Dependency Analysis & Graph Construction

This phase implements the static analysis part of the design.

1.  **Modify `SimulationEngine`**: Add a new method to the engine to build the communication graph. This should be called before the simulation starts.
2.  **Graph Representation**: Use a library like `petgraph` or a simple `HashMap<ComponentId, Vec<ComponentId>>` to store the graph. Nodes are components, and an edge `A -> B` exists if `A` can send an event that `B` subscribes to.
3.  **Engine Logic**:
    *   Iterate through all registered components.
    *   For each component, get its list of subscriptions.
    *   For every other component, check if it can emit events matching those subscriptions.
    *   If a match is found, add a directed edge to the graph.

## Phase 2: Profiling & Graph Weighting

This phase implements the dynamic analysis by running a short, single-threaded simulation to gather communication statistics.

1.  **Add Profiling Hook**: Augment the `SimulationEngine`'s event dispatch mechanism.
2.  **Event Counting**: Before dispatching an event, check the source and destination components. Increment a counter for that specific `(source_id, destination_id)` pair. A `HashMap<(ComponentId, ComponentId), u64>` is suitable for storing these counts.
3.  **Weight the Graph**: After the profiling run completes, iterate over the edges of the graph built in Phase 1. Assign the count from the profiling run as the weight for each edge. If an edge has no traffic, its weight is 0.

## Phase 3: Graph Partitioning

This phase uses the weighted graph to assign components to threads.

1.  **Integrate Partitioning Library**: Use the `metis` library added in Phase 0.
2.  **Create a `Partitioner` Struct**: This struct will take the weighted graph and the desired number of threads (`N`) as input. <!-- Phase 1 completed: We've added emitted_events method to BaseComponent, created DependencyGraph, updated SimulationEngine to build and store the graph, and added get_all_components to EventManager -->
3.  **Partition the Graph**: Call the `metis` library to partition the graph nodes into `N` parts, minimizing the total weight of cut edges.
4.  **Produce the Assignment Map**: The output of this phase must be a simple, efficient lookup map: `partition_map: HashMap<ComponentId, ThreadId>`. This map will be the definitive guide for where each component lives.

## Phase 4: Multi-threaded Execution Framework

This phase modifies the `SimulationEngine` to orchestrate the parallel run.

1.  **Update `SimulationEngine::run()`**: The main `run` method will now encapsulate the entire pipeline:
    *   Build graph (Phase 1).
    *   Run profiling (Phase 2).
    *   Weight graph (Phase 2).
    *   Partition graph (Phase 3).
    *   **Spawn and manage worker threads.**

2.  **Thread Spawning**:
    *   Create `N` worker threads.
    *   For each thread, create a dedicated `EventScheduler` instance.
    *   Create the communication channels. Each thread's scheduler gets the `Receiver` end of its own channel. The `SimulationEngine` will hold onto all the `Sender` ends to distribute them.
    *   Move the assigned components and the scheduler into the thread.

3.  **Inter-Thread Communication Setup**:
    *   The engine will create a `HashMap<ThreadId, crossbeam_channel::Sender<CrossThreadMessage>>` to allow any thread to send a message to any other thread.
    *   Each thread will receive a clone of this map, allowing it to communicate with its peers.

## Phase 5: Implementing Conservative Synchronization

This is the most complex phase, implementing the core time-management logic within each thread's `EventScheduler`.

1.  **Modify `EventScheduler`**: The scheduler needs new internal state:
    *   `thread_id: ThreadId`
    *   `incoming_guarantees: HashMap<ThreadId, Time>`: Stores the latest lookahead guarantee received from each other thread. Initialized to `Time=0`.
    *   `peer_senders: HashMap<ThreadId, Sender<CrossThreadMessage>>`: For talking to other threads.
    *   `event_receiver: Receiver<CrossThreadMessage>`: For receiving messages.

2.  **The Per-Thread Simulation Loop**: The main loop inside each `EventScheduler` will be:
    a. **Calculate Safe Time**: Determine the `safe_to_process_until` time. This is the `min()` of all values in the `incoming_guarantees` map.
    b. **Process Local Events**: Process all events in the local event queue with a timestamp `<= safe_to_process_until`.
    c. **Process Incoming Messages**: After processing local events, check the MPSC channel for new messages using a non-blocking `try_recv`.
        *   For `CrossThreadMessage::Event`, add the event to the local queue and update the `incoming_guarantees` for the source thread.
        *   For `CrossThreadMessage::Null`, simply update the `incoming_guarantees`.
    d. **Send NULL Message Updates**: If the scheduler's own clock has advanced, it must calculate its new lookahead guarantee (`current_time + L_thread_min`) and broadcast a `NullMessage` to all other threads it can send events to. This is crucial to prevent deadlocks.
    e. **Termination Condition**: The simulation ends when all threads report that their event queues are empty and no more events are in flight.

## Phase 6: Component-Level Integration

This phase integrates the lookahead concept into the user-facing component trait.

1.  **Modify `BaseComponent` Trait**:
    *   Add the optional `lookahead` method.
    *   Provide a default implementation that returns `TimeDelta::zero()`. This ensures backward compatibility and enforces the "safe default" principle.

    ```rust
    // In src/component.rs
    pub trait BaseComponent {
        // ... existing methods ...

        /// Declares the component's minimum time between emitted events.
        /// The engine uses this to optimize parallel execution.
        /// If not overridden, it defaults to zero, implying no guarantee.
        fn lookahead(&self) -> TimeDelta {
            TimeDelta::zero()
        }
    }
    ```

2.  **Engine Integration**: When a thread's scheduler calculates its outgoing lookahead guarantee, it will do so by finding the minimum lookahead among all the components it manages.

## Phase 7: Testing & Validation

1.  **Correctness Testing**:
    *   Create several test simulations with known event patterns and inter-component dependencies.
    *   Run each simulation in single-threaded mode and capture the sequence of events as the "ground truth".
    *   Run the same simulation in parallel mode with 2, 4, and 8 threads.
    *   Assert that the final state and the sequence of processed events are identical to the single-threaded run.

2.  **Performance Testing**:
    *   Create a large-scale simulation with many components and significant cross-thread communication.
    *   Benchmark the single-threaded performance.
    *   Benchmark the parallel performance with varying `lookahead` values (some 0, some non-zero) to validate that lookahead provides a speedup.

3.  **Deadlock Testing**:
    *   Design a scenario that would be prone to deadlock (e.g., a circular dependency of threads waiting on each other).
    *   Verify that the `NULL` message mechanism correctly breaks the deadlock and allows the simulation to proceed. 