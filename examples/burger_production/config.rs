use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductionMode {
    BufferBased,
    OrderBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingDelays {
    pub frying_cycles: u64,
    pub baking_cycles: u64,
    pub assembly_cycles: u64,
}

impl Default for ProcessingDelays {
    fn default() -> Self {
        Self {
            frying_cycles: 10,
            baking_cycles: 8,
            assembly_cycles: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferCapacities {
    pub fried_meat_capacity: usize,
    pub cooked_bread_capacity: usize,
    pub assembly_capacity: usize,
}

impl Default for BufferCapacities {
    fn default() -> Self {
        Self {
            fried_meat_capacity: 5,
            cooked_bread_capacity: 5,
            assembly_capacity: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderGenerationConfig {
    pub order_interval_cycles: u64,
    pub min_quantity_per_order: u32,
    pub max_quantity_per_order: u32,
}

impl Default for OrderGenerationConfig {
    fn default() -> Self {
        Self {
            order_interval_cycles: 15,
            min_quantity_per_order: 1,
            max_quantity_per_order: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurgerSimulationConfig {
    pub production_mode: ProductionMode,
    pub processing_delays: ProcessingDelays,
    pub buffer_capacities: BufferCapacities,
    pub order_generation: OrderGenerationConfig,
    pub simulation_duration_cycles: u64,
    pub random_seed: Option<u64>,
    pub enable_metrics: bool,
    pub enable_event_logging: bool,
}

impl Default for BurgerSimulationConfig {
    fn default() -> Self {
        Self {
            production_mode: ProductionMode::BufferBased,
            processing_delays: ProcessingDelays::default(),
            buffer_capacities: BufferCapacities::default(),
            order_generation: OrderGenerationConfig::default(),
            simulation_duration_cycles: 1000,
            random_seed: Some(42),
            enable_metrics: true,
            enable_event_logging: false,
        }
    }
}

impl BurgerSimulationConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_production_mode(mut self, mode: ProductionMode) -> Self {
        self.production_mode = mode;
        self
    }

    pub fn with_simulation_duration(mut self, cycles: u64) -> Self {
        self.simulation_duration_cycles = cycles;
        self
    }

    pub fn with_random_seed(mut self, seed: Option<u64>) -> Self {
        self.random_seed = seed;
        self
    }

    pub fn with_order_interval(mut self, cycles: u64) -> Self {
        self.order_generation.order_interval_cycles = cycles;
        self
    }

    pub fn with_order_quantity_range(mut self, min: u32, max: u32) -> Self {
        self.order_generation.min_quantity_per_order = min;
        self.order_generation.max_quantity_per_order = max;
        self
    }

    pub fn with_buffer_capacities(mut self, capacity: usize) -> Self {
        self.buffer_capacities.fried_meat_capacity = capacity;
        self.buffer_capacities.cooked_bread_capacity = capacity;
        self.buffer_capacities.assembly_capacity = capacity;
        self
    }

    pub fn with_metrics_enabled(mut self, enabled: bool) -> Self {
        self.enable_metrics = enabled;
        self
    }

    pub fn with_event_logging_enabled(mut self, enabled: bool) -> Self {
        self.enable_event_logging = enabled;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.simulation_duration_cycles == 0 {
            return Err("Simulation duration must be greater than 0".to_string());
        }

        if self.order_generation.order_interval_cycles == 0 {
            return Err("Order interval must be greater than 0".to_string());
        }

        if self.order_generation.min_quantity_per_order > self.order_generation.max_quantity_per_order {
            return Err("Min order quantity cannot be greater than max quantity".to_string());
        }

        if self.order_generation.min_quantity_per_order == 0 {
            return Err("Order quantity must be at least 1".to_string());
        }

        if self.buffer_capacities.fried_meat_capacity == 0 ||
           self.buffer_capacities.cooked_bread_capacity == 0 ||
           self.buffer_capacities.assembly_capacity == 0 {
            return Err("Buffer capacities must be greater than 0".to_string());
        }

        if self.processing_delays.frying_cycles == 0 ||
           self.processing_delays.baking_cycles == 0 ||
           self.processing_delays.assembly_cycles == 0 {
            return Err("Processing delays must be greater than 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BurgerSimulationConfig::default();
        assert_eq!(config.production_mode, ProductionMode::BufferBased);
        assert_eq!(config.processing_delays.frying_cycles, 10);
        assert_eq!(config.processing_delays.baking_cycles, 8);
        assert_eq!(config.processing_delays.assembly_cycles, 5);
        assert_eq!(config.buffer_capacities.fried_meat_capacity, 5);
        assert_eq!(config.simulation_duration_cycles, 1000);
        assert_eq!(config.random_seed, Some(42));
    }

    #[test]
    fn test_builder_pattern() {
        let config = BurgerSimulationConfig::new()
            .with_production_mode(ProductionMode::OrderBased)
            .with_simulation_duration(2000)
            .with_order_quantity_range(2, 8)
            .with_buffer_capacities(10);

        assert_eq!(config.production_mode, ProductionMode::OrderBased);
        assert_eq!(config.simulation_duration_cycles, 2000);
        assert_eq!(config.order_generation.min_quantity_per_order, 2);
        assert_eq!(config.order_generation.max_quantity_per_order, 8);
        assert_eq!(config.buffer_capacities.fried_meat_capacity, 10);
        assert_eq!(config.buffer_capacities.cooked_bread_capacity, 10);
        assert_eq!(config.buffer_capacities.assembly_capacity, 10);
    }

    #[test]
    fn test_validation() {
        let mut config = BurgerSimulationConfig::default();
        assert!(config.validate().is_ok());

        config.simulation_duration_cycles = 0;
        assert!(config.validate().is_err());

        config = BurgerSimulationConfig::default();
        config.order_generation.min_quantity_per_order = 10;
        config.order_generation.max_quantity_per_order = 5;
        assert!(config.validate().is_err());
    }
}