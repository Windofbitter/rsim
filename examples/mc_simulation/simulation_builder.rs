use rsim::core::builder::simulation_builder::Simulation;
use rsim::core::execution::config::SimulationConfig;
use rsim::core::types::{ComponentId, MemoryPort};
use crate::components::*;
use crate::components::component_states::*;
use crate::components::fifo_memory::FIFOMemory;

/// Delay mode configuration for simulation components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayMode {
    /// Components use random delays between min and max values
    Random,
    /// Components use fixed delays with configured values
    Fixed,
}

impl Default for DelayMode {
    fn default() -> Self {
        DelayMode::Random
    }
}

/// Configuration structure for McDonald's simulation components
#[derive(Debug, Clone)]
pub struct McSimulationConfig {
    /// Number of bakers (bread producers)
    pub num_bakers: usize,
    /// Number of fryers (meat producers)
    pub num_fryers: usize,
    /// Number of assemblers (burger assemblers)
    pub num_assemblers: usize,
    /// Number of customers (burger consumers)
    pub num_customers: usize,
    
    // Buffer capacity configurations
    /// Capacity for individual bread/meat buffers
    pub individual_buffer_capacity: i64,
    /// Capacity for inventory aggregation buffers
    pub inventory_buffer_capacity: i64,
    /// Capacity for assembler ingredient buffers
    pub assembler_buffer_capacity: i64,
    /// Capacity for final burger buffer
    pub burger_buffer_capacity: i64,
    /// Capacity for customer buffers (if using customer manager)
    pub customer_buffer_capacity: i64,
    
    // Timing configurations (min_delay, max_delay)
    /// Baker timing parameters (min, max cycles)
    pub baker_timing: (u32, u32),
    /// Fryer timing parameters (min, max cycles)
    pub fryer_timing: (u32, u32),
    /// Assembler timing parameters (min, max cycles)
    pub assembler_timing: (u32, u32),
    /// Customer timing parameters (min, max cycles)
    pub customer_timing: (u32, u32),
    
    // Delay mode configuration
    /// Delay mode for all components (random or fixed)
    pub delay_mode: DelayMode,
    /// Fixed delay values when using DelayMode::Fixed
    pub fixed_delay_values: FixedDelayValues,
    
    // RNG seed bases for deterministic behavior
    /// Base seed for bakers (each baker gets base + index)
    pub baker_seed_base: u64,
    /// Base seed for fryers (each fryer gets base + index)
    pub fryer_seed_base: u64,
    /// Base seed for assemblers (each assembler gets base + index)
    pub assembler_seed_base: u64,
    /// Base seed for customers (each customer gets base + index)
    pub customer_seed_base: u64,
}

/// Fixed delay values for each component type when using DelayMode::Fixed
#[derive(Debug, Clone)]
pub struct FixedDelayValues {
    /// Fixed delay for bakers (cycles)
    pub baker_delay: u32,
    /// Fixed delay for fryers (cycles)
    pub fryer_delay: u32,
    /// Fixed delay for assemblers (cycles)
    pub assembler_delay: u32,
    /// Fixed delay for customers (cycles)
    pub customer_delay: u32,
}

impl Default for FixedDelayValues {
    fn default() -> Self {
        Self {
            baker_delay: 3,     // Middle of default range (2-5)
            fryer_delay: 5,     // Middle of default range (3-7)
            assembler_delay: 2, // Middle of default range (1-3)
            customer_delay: 3,  // Middle of default range (1-5)
        }
    }
}

impl Default for McSimulationConfig {
    fn default() -> Self {
        Self {
            num_bakers: 5,
            num_fryers: 5,
            num_assemblers: 5,
            num_customers: 5,
            
            individual_buffer_capacity: 10,
            inventory_buffer_capacity: 100,
            assembler_buffer_capacity: 5,
            burger_buffer_capacity: 50,
            customer_buffer_capacity: 8,
            
            baker_timing: (2, 5),
            fryer_timing: (3, 7),
            assembler_timing: (1, 3),
            customer_timing: (1, 5),
            
            delay_mode: DelayMode::default(),
            fixed_delay_values: FixedDelayValues::default(),
            
            baker_seed_base: 1000,
            fryer_seed_base: 2000,
            assembler_seed_base: 3000,
            customer_seed_base: 4000,
        }
    }
}

/// Container for all created simulation components and their IDs
#[derive(Debug)]
pub struct McSimulationComponents {
    // Component IDs
    pub bakers: Vec<ComponentId>,
    pub fryers: Vec<ComponentId>,
    pub assemblers: Vec<ComponentId>,
    pub customers: Vec<ComponentId>,
    
    // Manager component IDs
    pub bread_manager: ComponentId,
    pub meat_manager: ComponentId,
    pub assembler_manager: ComponentId,
    pub customer_manager: ComponentId,
    
    // State memory component IDs
    pub baker_states: Vec<ComponentId>,
    pub fryer_states: Vec<ComponentId>,
    pub assembler_states: Vec<ComponentId>,
    pub customer_states: Vec<ComponentId>,
    
    // Buffer memory component IDs
    pub bread_buffers: Vec<ComponentId>,
    pub meat_buffers: Vec<ComponentId>,
    pub assembler_buffers: Vec<ComponentId>,
    pub assembler_output_buffers: Vec<ComponentId>,
    pub customer_buffers: Vec<ComponentId>,
    
    // Central buffer IDs
    pub bread_inventory_buffer: ComponentId,
    pub meat_inventory_buffer: ComponentId,
    pub burger_buffer: ComponentId,
    
    // Shared delay configuration
    pub delay_config: ComponentId,
}

/// Builder for McDonald's simulation that handles component creation and connections
pub struct McSimulationBuilder {
    config: McSimulationConfig,
}

impl McSimulationBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: McSimulationConfig::default(),
        }
    }
    
    /// Create a new builder with custom configuration
    pub fn with_config(config: McSimulationConfig) -> Self {
        Self {
            config,
        }
    }
    
    /// Set the number of each component type
    pub fn component_counts(mut self, bakers: usize, fryers: usize, assemblers: usize, customers: usize) -> Self {
        self.config.num_bakers = bakers;
        self.config.num_fryers = fryers;
        self.config.num_assemblers = assemblers;
        self.config.num_customers = customers;
        self
    }
    
    /// Set buffer capacities
    pub fn buffer_capacities(mut self, individual: i64, inventory: i64, assembler: i64, burger: i64) -> Self {
        self.config.individual_buffer_capacity = individual;
        self.config.inventory_buffer_capacity = inventory;
        self.config.assembler_buffer_capacity = assembler;
        self.config.burger_buffer_capacity = burger;
        self
    }
    
    /// Set timing parameters for all component types
    pub fn timing_parameters(
        mut self, 
        baker_timing: (u32, u32),
        fryer_timing: (u32, u32),
        assembler_timing: (u32, u32),
        customer_timing: (u32, u32)
    ) -> Self {
        self.config.baker_timing = baker_timing;
        self.config.fryer_timing = fryer_timing;
        self.config.assembler_timing = assembler_timing;
        self.config.customer_timing = customer_timing;
        self
    }
    
    
    /// Set delay mode for all components
    pub fn with_delay_mode(mut self, mode: DelayMode) -> Self {
        self.config.delay_mode = mode;
        self
    }
    
    /// Set fixed delay values for when using DelayMode::Fixed
    pub fn with_fixed_delays(mut self, baker: u32, fryer: u32, assembler: u32, customer: u32) -> Self {
        self.config.fixed_delay_values = FixedDelayValues {
            baker_delay: baker,
            fryer_delay: fryer,
            assembler_delay: assembler,
            customer_delay: customer,
        };
        self
    }
    
    /// Build the complete simulation with all components and connections
    pub fn build(self) -> Result<(Simulation, McSimulationComponents), String> {
        let mut sim = Simulation::new();
        
        self.build_with_simulation(sim)
    }
    
    /// Build the complete simulation with a preconfigured simulation
    pub fn build_with_config(self, config: SimulationConfig) -> Result<(Simulation, McSimulationComponents), String> {
        let mut sim = Simulation::with_config(config);
        
        self.build_with_simulation(sim)
    }
    
    /// Build the complete simulation with all components and connections using provided simulation
    fn build_with_simulation(self, mut sim: Simulation) -> Result<(Simulation, McSimulationComponents), String> {
        
        // =========================
        // 1. PRODUCTION COMPONENTS
        // =========================
        
        // Create bakers
        let mut bakers = Vec::new();
        for i in 0..self.config.num_bakers {
            let baker = sim.add_component(Baker::with_delay_config(
                self.config.baker_timing.0,
                self.config.baker_timing.1,
                self.config.fixed_delay_values.baker_delay,
                self.config.delay_mode,
                self.config.baker_seed_base + i as u64
            ));
            bakers.push(baker);
        }
        
        // Create baker state memory
        let mut baker_states = Vec::new();
        for _ in 0..self.config.num_bakers {
            let state = sim.add_memory_component(BakerState::new());
            baker_states.push(state);
        }
        
        // Create fryers
        let mut fryers = Vec::new();
        for i in 0..self.config.num_fryers {
            let fryer = sim.add_component(Fryer::new(
                self.config.fryer_timing.0,
                self.config.fryer_timing.1,
                self.config.fryer_seed_base + i as u64
            ));
            fryers.push(fryer);
        }
        
        // Create fryer state memory
        let mut fryer_states = Vec::new();
        for _ in 0..self.config.num_fryers {
            let state = sim.add_memory_component(FryerState::new());
            fryer_states.push(state);
        }
        
        // =========================
        // 2. INDIVIDUAL BUFFERS
        // =========================
        
        // Create bread buffers
        let mut bread_buffers = Vec::new();
        for _ in 0..self.config.num_bakers {
            let buffer = sim.add_memory_component(FIFOMemory::new(self.config.individual_buffer_capacity));
            bread_buffers.push(buffer);
        }
        
        // Create meat buffers
        let mut meat_buffers = Vec::new();
        for _ in 0..self.config.num_fryers {
            let buffer = sim.add_memory_component(FIFOMemory::new(self.config.individual_buffer_capacity));
            meat_buffers.push(buffer);
        }
        
        // =========================
        // 3. MANAGER COMPONENTS
        // =========================
        
        let bread_manager = sim.add_component(BreadManager::new());
        let meat_manager = sim.add_component(MeatManager::new());
        let assembler_manager = sim.add_component(AssemblerManager::new());
        
        // Create inventory buffers
        let bread_inventory_buffer = sim.add_memory_component(FIFOMemory::new(self.config.inventory_buffer_capacity));
        let meat_inventory_buffer = sim.add_memory_component(FIFOMemory::new(self.config.inventory_buffer_capacity));
        
        // Create shared delay configuration
        let delay_config = sim.add_memory_component(DelayConfig::new(
            self.config.delay_mode,
            self.config.fixed_delay_values.clone(),
            self.config.baker_timing,
            self.config.fryer_timing,
            self.config.assembler_timing,
            self.config.customer_timing,
        ));
        
        // =========================
        // 4. ASSEMBLY COMPONENTS
        // =========================
        
        // Create assembler buffers
        let mut assembler_buffers = Vec::new();
        for _ in 0..self.config.num_assemblers {
            let buffer = sim.add_memory_component(FIFOMemory::new(self.config.assembler_buffer_capacity));
            assembler_buffers.push(buffer);
        }
        
        // Create assemblers
        let mut assemblers = Vec::new();
        for i in 0..self.config.num_assemblers {
            let assembler = sim.add_component(Assembler::new(
                self.config.assembler_timing.0,
                self.config.assembler_timing.1,
                self.config.assembler_seed_base + i as u64
            ));
            assemblers.push(assembler);
        }
        
        // Create assembler state memory
        let mut assembler_states = Vec::new();
        for _ in 0..self.config.num_assemblers {
            let state = sim.add_memory_component(AssemblerState::new());
            assembler_states.push(state);
        }
        
        // Create individual assembler output buffers (always used)
        let mut assembler_output_buffers = Vec::new();
        for _ in 0..self.config.num_assemblers {
            let buffer = sim.add_memory_component(FIFOMemory::new(self.config.burger_buffer_capacity));
            assembler_output_buffers.push(buffer);
        }
        
        // Create burger buffer (kept for legacy compatibility, but not used in customer manager mode)
        let burger_buffer = sim.add_memory_component(FIFOMemory::new(self.config.burger_buffer_capacity));
        
        // =========================
        // 5. CONSUMER COMPONENTS
        // =========================
        
        // Always create customer manager
        let customer_manager = sim.add_component(CustomerManager::new());
        
        // Create customer buffers (always used)
        let mut customer_buffers = Vec::new();
        for _ in 0..self.config.num_customers {
            let buffer = sim.add_memory_component(FIFOMemory::new(self.config.customer_buffer_capacity));
            customer_buffers.push(buffer);
        }
        
        // Create customers
        let mut customers = Vec::new();
        for i in 0..self.config.num_customers {
            let customer = sim.add_component(Customer::new(
                self.config.customer_timing.0,
                self.config.customer_timing.1,
                self.config.customer_seed_base + i as u64
            ));
            customers.push(customer);
        }
        
        // Create customer state memory
        let mut customer_states = Vec::new();
        for _ in 0..self.config.num_customers {
            let state = sim.add_memory_component(CustomerState::new());
            customer_states.push(state);
        }
        
        // =========================
        // 6. MEMORY CONNECTIONS
        // =========================
        
        // Connect bakers to bread buffers (1:1)
        for i in 0..self.config.num_bakers {
            sim.connect_memory_port(bakers[i].memory_port("bread_buffer"), bread_buffers[i].clone())?;
            sim.connect_memory_port(bakers[i].memory_port("baker_state"), baker_states[i].clone())?;
        }
        
        // Connect fryers to meat buffers (1:1)
        for i in 0..self.config.num_fryers {
            sim.connect_memory_port(fryers[i].memory_port("meat_buffer"), meat_buffers[i].clone())?;
            sim.connect_memory_port(fryers[i].memory_port("fryer_state"), fryer_states[i].clone())?;
        }
        
        // Connect bread buffers to bread manager (N:1)
        for i in 0..self.config.num_bakers {
            sim.connect_memory_port(bread_manager.memory_port(&format!("bread_buffer_{}", i + 1)), bread_buffers[i].clone())?;
        }
        
        // Connect meat buffers to meat manager (N:1)
        for i in 0..self.config.num_fryers {
            sim.connect_memory_port(meat_manager.memory_port(&format!("meat_buffer_{}", i + 1)), meat_buffers[i].clone())?;
        }
        
        // Connect managers to inventory buffers
        sim.connect_memory_port(bread_manager.memory_port("bread_inventory_out"), bread_inventory_buffer.clone())?;
        sim.connect_memory_port(meat_manager.memory_port("meat_inventory_out"), meat_inventory_buffer.clone())?;
        
        // Connect assembler manager to inventory buffers
        sim.connect_memory_port(assembler_manager.memory_port("bread_inventory"), bread_inventory_buffer.clone())?;
        sim.connect_memory_port(assembler_manager.memory_port("meat_inventory"), meat_inventory_buffer.clone())?;
        
        // Connect assembler manager to assembler buffers (1:N)
        for i in 0..self.config.num_assemblers {
            sim.connect_memory_port(assembler_manager.memory_port(&format!("assembler_buffer_{}", i + 1)), assembler_buffers[i].clone())?;
        }
        
        // Connect assemblers to their buffers and individual output buffers
        for i in 0..self.config.num_assemblers {
            sim.connect_memory_port(assemblers[i].memory_port("ingredient_buffer"), assembler_buffers[i].clone())?;
            
            // Always connect to individual output buffers (customer manager mode)
            sim.connect_memory_port(assemblers[i].memory_port("burger_buffer"), assembler_output_buffers[i].clone())?;
            
            sim.connect_memory_port(assemblers[i].memory_port("assembler_state"), assembler_states[i].clone())?;
        }
        
        // Connect customers using customer manager (always used)
        // Connect customer manager to individual assembler output buffers
        for i in 0..self.config.num_assemblers {
            sim.connect_memory_port(customer_manager.memory_port(&format!("assembler_output_{}", i + 1)), assembler_output_buffers[i].clone())?;
        }
        
        // Connect customer manager to customer buffers and customers to their buffers
        for i in 0..self.config.num_customers {
            sim.connect_memory_port(customer_manager.memory_port(&format!("customer_buffer_{}", i + 1)), customer_buffers[i].clone())?;
            sim.connect_memory_port(customers[i].memory_port("burger_buffer"), customer_buffers[i].clone())?;
        }
        
        // Connect customer state memory
        for i in 0..self.config.num_customers {
            sim.connect_memory_port(customers[i].memory_port("customer_state"), customer_states[i].clone())?;
        }
        
        // Create component container
        let components = McSimulationComponents {
            bakers,
            fryers,
            assemblers,
            customers,
            bread_manager,
            meat_manager,
            assembler_manager,
            customer_manager,
            baker_states,
            fryer_states,
            assembler_states,
            customer_states,
            bread_buffers,
            meat_buffers,
            assembler_buffers,
            assembler_output_buffers,
            customer_buffers,
            bread_inventory_buffer,
            meat_inventory_buffer,
            burger_buffer,
            delay_config,
        };
        
        Ok((sim, components))
    }
}

/// Convenience function to build a McDonald's simulation with specified component counts
pub fn build_mc_simulation_config(
    num_bakers: usize,
    num_fryers: usize,
    num_assemblers: usize,
    num_customers: usize
) -> Result<(Simulation, McSimulationComponents), String> {
    McSimulationBuilder::new()
        .component_counts(num_bakers, num_fryers, num_assemblers, num_customers)
        .build()
}

/// Convenience function to build a small-scale McDonald's simulation (default settings)
pub fn build_small_mc_simulation() -> Result<(Simulation, McSimulationComponents), String> {
    build_mc_simulation_config(3, 3, 3, 3)
}

/// Convenience function to build a medium-scale McDonald's simulation 
pub fn build_medium_mc_simulation() -> Result<(Simulation, McSimulationComponents), String> {
    build_mc_simulation_config(5, 5, 5, 5)
}

/// Convenience function to build a large-scale McDonald's simulation
pub fn build_large_mc_simulation() -> Result<(Simulation, McSimulationComponents), String> {
    build_mc_simulation_config(10, 10, 10, 10)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = McSimulationConfig::default();
        assert_eq!(config.num_bakers, 5);
        assert_eq!(config.num_fryers, 5);
        assert_eq!(config.num_assemblers, 5);
        assert_eq!(config.num_customers, 5);
    }
    
    #[test]
    fn test_builder_component_counts() {
        let builder = McSimulationBuilder::new()
            .component_counts(2, 3, 4, 5);
        assert_eq!(builder.config.num_bakers, 2);
        assert_eq!(builder.config.num_fryers, 3);
        assert_eq!(builder.config.num_assemblers, 4);
        assert_eq!(builder.config.num_customers, 5);
    }
    
    #[test]
    fn test_build_small_simulation() {
        let result = build_small_mc_simulation();
        assert!(result.is_ok());
        let (_sim, components) = result.unwrap();
        assert_eq!(components.bakers.len(), 3);
        assert_eq!(components.fryers.len(), 3);
        assert_eq!(components.assemblers.len(), 3);
        assert_eq!(components.customers.len(), 3);
    }
}