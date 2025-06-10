# Burger Production Event-Based Design

This document defines the event-based architecture for the burger production simulation, mapping the physical process to our event-driven framework.

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

## Events

### Production Events
- **`start_frying`**
  - Source: Fryer
  - Target: Fryer (self-scheduled)
  - Data: `ComponentValue::Int(1)` (quantity)
  - Triggers meat processing cycle

- **`meat_ready`**
  - Source: Fryer
  - Target: FriedMeatBuffer
  - Data: `ComponentValue::Int(1)` (fried patty)
  - Signals completion of meat frying

- **`start_baking`**
  - Source: Baker
  - Target: Baker (self-scheduled)
  - Data: `ComponentValue::Int(1)` (quantity)
  - Triggers bread processing cycle

- **`bread_ready`**
  - Source: Baker
  - Target: CookedBreadBuffer
  - Data: `ComponentValue::Int(1)` (cooked bun)
  - Signals completion of bread baking

- **`start_assembly`**
  - Source: Assembler
  - Target: Assembler (self-scheduled)
  - Data: `ComponentValue::Int(1)` (quantity)
  - Triggers burger assembly cycle

- **`burger_ready`**
  - Source: Assembler
  - Target: AssemblyBuffer
  - Data: `ComponentValue::Int(1)` (complete burger)
  - Signals completion of burger assembly

### Buffer Events
- **`item_added`**
  - Source: FIFO Buffer
  - Target: Downstream consumers
  - Data: `ComponentValue::Int(current_count)`
  - Notifies when item becomes available

- **`buffer_full`**
  - Source: FIFO Buffer
  - Target: Upstream producers
  - Data: `ComponentValue::Bool(true)`
  - Signals backpressure - stop production

- **`buffer_space_available`**
  - Source: FIFO Buffer
  - Target: Upstream producers
  - Data: `ComponentValue::Bool(false)`
  - Signals production can resume

- **`request_item`**
  - Source: Consumer
  - Target: FIFO Buffer
  - Data: `ComponentValue::Int(quantity)`
  - Requests items from buffer

### Demand Events
- **`generate_order`**
  - Source: Client
  - Target: Client (self-scheduled)
  - Data: `ComponentValue::Int(order_size)`
  - Periodic order generation

- **`place_order`**
  - Source: Client
  - Target: AssemblyBuffer
  - Data: `ComponentValue::Int(burger_count)`
  - Requests burgers from final buffer

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

This event-based design ensures deterministic execution, proper backpressure handling, and realistic simulation of the burger production process.