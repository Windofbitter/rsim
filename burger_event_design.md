# Burger Production Trait-Based Event Design

This document defines the trait-based event architecture for the burger production simulation, mapping the physical process to our event-driven framework with type-safe events.

## Components

### Production Components
- **Fryer** (`ComponentId: "fryer"`)
  - Processes raw meat into fried patties
  - Processing time: configurable delay
  - Capacity: single item processing

- **Baker** (`ComponentId: "baker"`)  
  - Processes raw bread into cooked buns
  - Processing time: configurable delay
  - Capacity: single item processing

- **Assembler** (`ComponentId: "assembler"`)
  - Combines fried meat + cooked bread → complete burger
  - Processing time: configurable delay
  - Requires both ingredients to proceed

### Buffer Components (FIFO)
- **FriedMeatBuffer** (`ComponentId: "fried_meat_fifo"`)
  - Stores fried meat patties
  - Max capacity: configurable
  - FIFO ordering

- **CookedBreadBuffer** (`ComponentId: "cooked_bread_fifo"`)
  - Stores cooked buns
  - Max capacity: configurable
  - FIFO ordering

- **AssemblyBuffer** (`ComponentId: "assembly_fifo"`)
  - Stores completed burgers
  - Max capacity: configurable
  - FIFO ordering

### Demand Component
- **Client** (`ComponentId: "client"`)
  - Generates burger orders periodically
  - Order quantity: normal distribution
  - Consumes from assembly buffer

## Event Types

All events implement the `Event` trait with type-safe data fields:

```rust
pub trait Event: Debug + Clone {
    fn event_type(&self) -> &'static str;
    fn source_id(&self) -> &ComponentId;
    fn target_ids(&self) -> Option<&[ComponentId]>;
    fn event_id(&self) -> &EventId;
}
```

### Production Event Structs

```rust
#[derive(Debug, Clone)]
pub struct StartFryingEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct MeatReadyEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct StartBakingEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct BreadReadyEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct StartAssemblyEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct BurgerReadyEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
    pub quantity: u32,
}
```

### Buffer Event Structs

```rust
#[derive(Debug, Clone)]
pub struct ItemAddedEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
    pub current_count: u32,
    pub item_type: String,
}

#[derive(Debug, Clone)]
pub struct BufferFullEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
    pub capacity: u32,
}

#[derive(Debug, Clone)]
pub struct BufferSpaceAvailableEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
    pub available_space: u32,
}

#[derive(Debug, Clone)]
pub struct RequestItemEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
    pub quantity: u32,
}
```

### Demand Event Structs

```rust
#[derive(Debug, Clone)]
pub struct GenerateOrderEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
    pub order_size: u32,
}

#[derive(Debug, Clone)]
pub struct PlaceOrderEvent {
    pub id: EventId,
    pub source_id: ComponentId,
    pub target_ids: Option<Vec<ComponentId>>,
    pub burger_count: u32,
}
```

## Event Flow

### Initialization
1. All components register with EventManager
2. Fryer schedules initial `start_frying` event
3. Baker schedules initial `start_baking` event
4. Client schedules initial `generate_order` event

### Production Cycle
1. **Fryer**: `start_frying` → process delay → `meat_ready` → FriedMeatBuffer
2. **Baker**: `start_baking` → process delay → `bread_ready` → CookedBreadBuffer  
3. **Assembler**: Waits for both ingredients → `start_assembly` → process delay → `burger_ready`

### Backpressure Control
1. Buffer sends `buffer_full` when at capacity
2. Producer stops scheduling new work events
3. Buffer sends `buffer_space_available` when space opens
4. Producer resumes scheduling work events

### Demand Processing
1. Client generates `generate_order` periodically
2. Client sends `place_order` to AssemblyBuffer
3. AssemblyBuffer fulfills orders if inventory available
4. Unfulfilled orders remain as pending demand

## Component Subscriptions

Components subscribe to event types by their static string identifiers:

```rust
// Fryer subscriptions
["start_frying", "buffer_full", "buffer_space_available"]

// Baker subscriptions  
["start_baking", "buffer_full", "buffer_space_available"]

// Assembler subscriptions
["start_assembly", "item_added", "buffer_full", "buffer_space_available"]

// FIFO Buffer subscriptions
["meat_ready", "bread_ready", "burger_ready", "request_item", "place_order"]

// Client subscriptions
["generate_order", "item_added"]
```

## Event Type Constants

```rust
pub const START_FRYING: &str = "start_frying";
pub const MEAT_READY: &str = "meat_ready";
pub const START_BAKING: &str = "start_baking";
pub const BREAD_READY: &str = "bread_ready";
pub const START_ASSEMBLY: &str = "start_assembly";
pub const BURGER_READY: &str = "burger_ready";
pub const ITEM_ADDED: &str = "item_added";
pub const BUFFER_FULL: &str = "buffer_full";
pub const BUFFER_SPACE_AVAILABLE: &str = "buffer_space_available";
pub const REQUEST_ITEM: &str = "request_item";
pub const GENERATE_ORDER: &str = "generate_order";
pub const PLACE_ORDER: &str = "place_order";
```

This trait-based event design ensures type safety, deterministic execution, proper backpressure handling, and realistic simulation of the burger production process.