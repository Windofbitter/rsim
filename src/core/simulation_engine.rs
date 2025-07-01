// Remove old imports for EventManager and EventScheduler
use super::connection_manager::ConnectionManager;
use super::cycle_engine::CycleEngine;

pub struct SimulationEngine {
    cycle_engine: CycleEngine,
    max_cycles: Option<u64>,
}

impl SimulationEngine {
    pub fn new(connection_manager: ConnectionManager, max_cycles: Option<u64>) -> Result<Self, String> {
        let mut engine = Self {
            cycle_engine: CycleEngine::new(connection_manager),
            max_cycles,
        };
        
        // Build the required mappings before simulation can run
        engine.cycle_engine.connection_manager.build_evaluation_order()?;
        engine.cycle_engine.connection_manager.build_input_mapping();
        
        Ok(engine)
    }

    pub fn run(&mut self) -> Result<u64, String> {
        while self.max_cycles.map_or(true, |max| self.current_cycle() < max) {
            self.step()?;
        }
        Ok(self.current_cycle())
    }

    pub fn step(&mut self) -> Result<bool, String> {
        self.cycle_engine.run_cycle();
        Ok(true)
    }

    pub fn current_cycle(&self) -> u64 {
        self.cycle_engine.current_cycle
    }
}