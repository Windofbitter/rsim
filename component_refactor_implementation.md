# Component Refactor Implementation Guide

## Overview
This document provides step-by-step instructions for implementing the component architecture refactor described in `component_refactor_design.md`. The refactor transitions from trait-based components to a module-to-component pattern for better performance and maintainability.

## Implementation Strategy

### Phase 1: Foundation - New Core Types
Create the foundational types and traits without breaking existing functionality.

**Step 1.1: Create `core/state.rs`**
- Implement `ComponentState` trait with `as_any()` and `as_any_mut()` methods
- Implement `MemoryData` marker trait
- Add basic error types for state management

**Step 1.2: Create `core/component_module.rs`**
- Define `ComponentModule` enum (Processing, Memory, Probe variants)
- Implement `ProcessorModule` struct with `PortSpec` and `EvaluationContext`
- Implement `ProbeModule` struct with probe function types
- Implement `MemoryModuleTrait` and `MemoryModule<T>` for type-erased memory modules
- Add convenience methods for `PortSpec` construction

**Step 1.3: Create `core/component_manager.rs`**
- Implement `ComponentManager` struct with module registry
- Add `register_module()` method for storing component templates
- Add `create_component()` and `create_component_auto_id()` methods
- Implement unique ID generation logic

### Phase 2: Integration Layer
Create the bridge between new and existing systems.

**Step 2.1: Update `core/component.rs`**
- Add new unified `Component` struct alongside existing traits
- Keep existing trait definitions (`BaseComponent`, `ProcessingComponent`, etc.) for backward compatibility
- Add `ComponentType` enum to distinguish component types
- Update `Event` type alias and `MemoryError` enum as needed

**Step 2.2: Update `core/memory_proxy.rs`**
- Extend `EngineMemoryProxy` trait with new type-safe methods
- Add `read<T: MemoryData>()` method with downcasting
- Add `write<T: MemoryData>()` method with type checking
- Maintain backward compatibility with existing address-based methods

**Step 2.3: Create `core/simulation_builder.rs`**
- Implement `SimulationBuilder` struct for fluent API
- Add methods: `create_component()`, `connect()`, `connect_memory()`, `connect_probe()`
- Implement validation methods: `validate_connections()`, `validate_required_ports()`
- Add `build()` method that creates a `CycleEngine`

### Phase 3: Engine Integration
Update the simulation engine to work with the new component system.

**Step 3.1: Update `core/cycle_engine.rs`**
- Add support for new `Component` instances alongside existing trait objects
- Modify `run_cycle()` to handle both old and new component types
- Create new execution paths for module-based components using `EvaluationContext`
- Maintain backward compatibility with existing component registration methods

**Step 3.2: Update `core/connection_manager.rs`**
- Add methods to work with new component port specifications
- Implement port validation using `PortSpec` metadata
- Add support for fluent connection API from `SimulationBuilder`
- Keep existing connection methods for backward compatibility

**Step 3.3: Update `core/component_registry.rs`**
- Add storage for new `Component` instances
- Implement methods to query components by type and ID
- Add conversion utilities between old and new component types
- Maintain existing storage for trait objects during migration

### Phase 4: Testing and Validation
Ensure the new system works correctly and maintains backward compatibility.

**Step 4.1: Create Test Infrastructure**
- Add unit tests for `ComponentManager`, `ComponentModule`, and `SimulationBuilder`
- Create integration tests comparing old vs new component behavior
- Add performance benchmarks to validate performance improvements
- Test error handling and validation logic

**Step 4.2: Create Migration Examples**
- Convert existing components (from `src/components/`) to use new module system
- Create examples showing how to migrate from trait-based to module-based components
- Document the migration process for users

**Step 4.3: Update Documentation**
- Update `CLAUDE.md` with new component creation patterns
- Add migration guide for existing users
- Document the new fluent API for component creation and connection

### Phase 5: Migration and Cleanup
Gradually migrate existing code and remove deprecated functionality.

**Step 5.1: Migrate Existing Components**
- Convert components in `src/components/` to use new module system
- Update example code and documentation
- Ensure all tests pass with new implementation

**Step 5.2: Performance Optimization**
- Profile the new system vs old system
- Optimize hot paths in component evaluation
- Remove unnecessary allocations in `EvaluationContext`

**Step 5.3: Deprecation and Cleanup**
- Mark old trait-based methods as deprecated
- Add migration warnings for users of old API
- Plan removal of deprecated functionality in future version

## Key Implementation Notes

### Error Handling
- Use `Result<T, String>` for component creation and registration errors
- Implement detailed error messages for debugging
- Add validation at component creation time, not runtime

### Performance Considerations
- Use function pointers instead of trait objects where possible
- Minimize allocations in `EvaluationContext` during component evaluation
- Cache port specifications and avoid repeated lookups

### Type Safety
- Leverage Rust's type system for compile-time safety where possible
- Use runtime type checking only when necessary (e.g., memory component downcasting)
- Provide clear error messages for type mismatches

### Backward Compatibility
- Keep existing APIs working during migration period
- Provide conversion utilities between old and new systems
- Document breaking changes clearly

## Testing Strategy

### Unit Tests
- Test component module registration and creation
- Test port specification validation
- Test memory component type safety
- Test simulation builder fluent API

### Integration Tests
- Test complete simulation runs with new component system
- Compare results between old and new implementations
- Test mixed usage of old and new components during migration

### Performance Tests
- Benchmark component creation and evaluation performance
- Compare memory usage between old and new systems
- Test with large numbers of component instances

## Success Criteria
- All existing functionality works with new implementation
- Performance improvements are measurable
- API is more ergonomic and easier to use
- Migration path is clear and well-documented
- Test coverage is maintained or improved