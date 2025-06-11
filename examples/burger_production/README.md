# Burger Production Events

This module contains all the event types used in the burger production simulation, organized by category.

## Event Categories

### Production Events (`production/`)
- **TriggerProductionEvent**: Initiates production cycle in a component
- **MeatReadyEvent**: Signals completion of a fried meat patty
- **BreadReadyEvent**: Signals completion of a cooked bun  
- **BurgerReadyEvent**: Signals completion of an assembled burger

### Buffer Management Events (`buffer_management/`)
- **ItemAddedEvent**: Broadcasts that an item was successfully added to a buffer
- **RequestItemEvent**: Requests an item from a buffer
- **BufferFullEvent**: Signals that a buffer has reached capacity
- **BufferSpaceAvailableEvent**: Signals that a full buffer now has space
- **ItemDroppedEvent**: Notifies that an item/order was rejected due to full buffer

### Demand Events (`demand/`)
- **GenerateOrderEvent**: Triggers order generation in the Client
- **PlaceOrderEvent**: Places a burger order
- **OrderFulfilledEvent**: Notifies that an order has been completed

### Future Events (`future/`)
For OrderBased production mode (future implementation):
- **ProductionRequestEvent**: Requests production from upstream component
- **InventoryQueryEvent**: Queries current inventory levels
- **InventoryStatusEvent**: Reports current inventory status

## Usage

Each event implements the base `Event` trait from the simulation engine and can be used with the component system. All events include:
- Unique event ID generation using UUID
- Type-safe data fields
- Source and target component routing
- HashMap data representation for the simulation engine

## Example

```rust
use rsim::examples::burger_production::production::MeatReadyEvent;

let event = MeatReadyEvent::new(
    "fryer_1".to_string(),
    Some(vec!["meat_buffer".to_string()]),
    "meat_patty_123".to_string(),
);
```