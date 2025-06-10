# Burger Production System Implementation Plan

This document outlines the step-by-step implementation plan for the burger production simulation using the trait-based event system. The implementation will be created as an example in the `examples/burger_production/` directory.

## Implementation Phases

### Phase 1: Event Infrastructure (Foundation) ✅ COMPLETED
**Goal**: Implement all burger-specific event types and constants

**Tasks**: ✅ ALL COMPLETED
1. ✅ Create `examples/burger_production/events/mod.rs` module
2. ✅ Create event struct implementations (split into multiple files for better organization)
3. ✅ Implement Event trait for each event type
4. ✅ Add event type constants
5. ✅ Create helper functions for event creation

**Files Created**:
- ✅ `examples/burger_production/events/mod.rs`
- ✅ `examples/burger_production/events/fryer_events.rs` (StartFryingEvent, MeatReadyEvent)
- ✅ `examples/burger_production/events/baker_events.rs` (StartBakingEvent, BreadReadyEvent)
- ✅ `examples/burger_production/events/assembler_events.rs` (StartAssemblyEvent, BurgerReadyEvent)
- ✅ `examples/burger_production/events/buffer_events.rs` (ItemAddedEvent, BufferFullEvent, BufferSpaceAvailableEvent, RequestItemEvent)
- ✅ `examples/burger_production/events/demand_events.rs` (GenerateOrderEvent, PlaceOrderEvent)

**Acceptance Criteria**: ✅ ALL MET
- ✅ All 10 event types implemented and compile without errors
- ✅ Each event implements the Event trait correctly with proper methods
- ✅ Event constants defined and accessible from components
- ✅ Proper data structure with HashMap<String, ComponentValue> for event data
- ✅ Target ID handling for directed events vs broadcast events

### Phase 2: FIFO Buffer Component (Core Reusable Component) ✅ COMPLETED
**Goal**: Implement generic FIFO buffer system with specialized buffer components

**Tasks**: ✅ ALL COMPLETED
1. ✅ Create `examples/burger_production/components/fifo_buffer.rs`
2. ✅ Implement GenericFifoBuffer struct with configurable capacity and item type
3. ✅ Add internal state management (queue, current_count, capacity, was_empty tracking)
4. ✅ Implement BaseComponent trait for specialized buffers
5. ✅ Create specialized buffer components with targeted event subscriptions:
   - FriedMeatBuffer: `["meat_ready", "request_item"]`
   - CookedBreadBuffer: `["bread_ready", "request_item"]`
   - AssemblyBuffer: `["burger_ready", "place_order"]`
6. ✅ Implement react_atomic logic for:
   - Item storage with type-specific handling
   - Item requests with pull-based consumption
   - Backpressure signaling (buffer_full, buffer_space_available)
   - Downstream notifications (item_added)

**Files Created**:
- ✅ `examples/burger_production/components/fifo_buffer.rs`
- ✅ `examples/burger_production/components/mod.rs`
- ✅ `examples/burger_production/lib.rs`

**Acceptance Criteria**: ✅ ALL MET
- ✅ Generic FIFO base with common functionality (GenericFifoBuffer)
- ✅ Three specialized buffer types: FriedMeatBuffer, CookedBreadBuffer, AssemblyBuffer
- ✅ Type-safe event filtering - each buffer only processes relevant events
- ✅ Buffer stores items in FIFO order with proper state change detection
- ✅ Backpressure events sent when full/space available using was_empty tracking
- ✅ ItemAddedEvent sent to downstream components with correct item_type
- ✅ Pull-based consumption via REQUEST_ITEM_EVENT and PLACE_ORDER_EVENT

### Phase 3: Production Components (Fryer, Baker, Assembler)
**Goal**: Implement the three production components with processing delays

**Tasks**:
1. Create `examples/burger_production/components/fryer.rs`
   - Subscribes to: `["start_frying", "buffer_full", "buffer_space_available"]`
   - Generates: `StartFryingEvent` (self-scheduled), `MeatReadyEvent`
   - Implements processing delay and backpressure handling

2. Create `examples/burger_production/components/baker.rs`
   - Subscribes to: `["start_baking", "buffer_full", "buffer_space_available"]`
   - Generates: `StartBakingEvent` (self-scheduled), `BreadReadyEvent`
   - Implements processing delay and backpressure handling

3. Create `examples/burger_production/components/assembler.rs`
   - Subscribes to: `["start_assembly", "item_added", "buffer_full", "buffer_space_available"]`
   - Generates: `StartAssemblyEvent` (self-scheduled), `BurgerReadyEvent`
   - Waits for both meat and bread availability
   - Coordinates with two input buffers

**Files to Create**:
- `examples/burger_production/components/fryer.rs`
- `examples/burger_production/components/baker.rs`
- `examples/burger_production/components/assembler.rs`

**Acceptance Criteria**:
- Each component processes items with configurable delays
- Proper backpressure handling (stop when downstream full)
- Self-scheduling for continuous production
- Assembler waits for both ingredients

### Phase 4: Client Component (Demand Generator)
**Goal**: Implement client that generates orders with normal distribution

**Tasks**:
1. Create `examples/burger_production/components/client.rs`
   - Subscribes to: `["generate_order", "item_added"]`
   - Generates: `GenerateOrderEvent` (self-scheduled), `PlaceOrderEvent`
   - Implements normal distribution for order sizes
   - Tracks pending orders vs fulfilled orders

**Files to Create**:
- `examples/burger_production/components/client.rs`

**Acceptance Criteria**:
- Orders generated with normal distribution
- Periodic order generation (self-scheduling)
- Order fulfillment tracking
- Integration with assembly buffer

### Phase 5: System Integration and Configuration
**Goal**: Wire all components together and create simulation setup

**Tasks**:
1. Create `examples/burger_production/main.rs` - main simulation setup
2. Create `examples/burger_production/lib.rs` - module exports
3. Implement component instantiation and configuration
4. Wire component relationships (which buffer connects to which producer/consumer)
5. Create simulation configuration struct
6. Add initialization sequence
7. Integrate with rsim SimulationEngine

**Files to Create**:
- `examples/burger_production/main.rs`
- `examples/burger_production/lib.rs`
- `examples/burger_production/Cargo.toml` (example-specific dependencies)

**Acceptance Criteria**:
- All components instantiated with proper IDs
- Event routing configured correctly
- Simulation runs end-to-end
- Configurable parameters (delays, capacities, distributions)

### Phase 6: Testing and Validation
**Goal**: Comprehensive testing of the burger production system

**Tasks**:
1. Unit tests for each component
2. Integration tests for component interactions
3. End-to-end simulation tests
4. Performance benchmarks
5. Backpressure scenario testing
6. Order fulfillment accuracy testing

**Files to Create**:
- `tests/burger_production_tests.rs`
- `benches/burger_simulation_bench.rs`

**Acceptance Criteria**:
- All unit tests pass
- Integration tests verify correct event flow
- Backpressure prevents buffer overflow
- Order fulfillment matches expected rates
- Performance benchmarks establish baseline

## Implementation Order

```
Phase 1: Events → Phase 2: FIFO Buffer → Phase 3: Production Components → Phase 4: Client → Phase 5: Integration → Phase 6: Testing
```

## Key Design Decisions

### Component State Management
- Each component maintains internal state (processing status, timers, counters)
- State updates happen atomically within `react_atomic()`
- No shared state between components - all communication via events

### Event Scheduling Strategy
- Self-scheduling events for continuous operations (fryer keeps frying)
- Processing delays implemented via event scheduling with future timestamps
- Backpressure handled by suspending self-scheduling events

### Buffer Management
- GenericFifoBuffer provides common FIFO functionality with composition pattern
- Three specialized buffer components: FriedMeatBuffer, CookedBreadBuffer, AssemblyBuffer
- Type-safe event filtering ensures each buffer only handles relevant events
- Item types distinguished by expected_item_type field for validation
- State change detection via was_empty field prevents event spam
- Capacity management prevents overflow with proper backpressure signaling

### Error Handling
- Invalid event types logged but don't crash simulation
- Buffer overflow prevented by backpressure
- Missing ingredients handled gracefully by assembler

## Configuration Parameters

```rust
pub struct BurgerSimulationConfig {
    // Processing delays (simulation time units)
    pub frying_delay: u64,
    pub baking_delay: u64,
    pub assembly_delay: u64,
    
    // Buffer capacities  
    pub meat_buffer_capacity: u32,
    pub bread_buffer_capacity: u32,
    pub assembly_buffer_capacity: u32,
    
    // Client behavior
    pub order_generation_interval: u64,
    pub order_size_mean: f64,
    pub order_size_std_dev: f64,
    
    // Simulation parameters
    pub max_simulation_cycles: u64,
}
```

## Example Cargo.toml Structure

The `examples/burger_production/Cargo.toml` will reference the main rsim library:

```toml
[package]
name = "burger_production"
version = "0.1.0"
edition = "2021"

[dependencies]
rsim = { path = "../.." }
rand = "0.8"
rand_distr = "0.4"

[[bin]]
name = "burger_production"
path = "main.rs"
```

## Running the Example

Once implemented, the burger production example can be run from the project root:

```bash
cd examples/burger_production
cargo run
```

This implementation plan ensures systematic development of a robust, type-safe burger production simulation that demonstrates the power of the trait-based event system while maintaining a clean separation as an example project.