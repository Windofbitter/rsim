use crate::core::execution::cycle_engine::CycleEngine;

pub struct SimulationEngine {
    cycle_engine: CycleEngine,
    max_cycles: Option<u64>,
}

impl SimulationEngine {
    pub fn new(
        mut cycle_engine: CycleEngine,
        max_cycles: Option<u64>,
    ) -> Result<Self, String> {
        // Build topological execution order for deterministic simulation
        cycle_engine.build_execution_order()?;

        let engine = Self {
            cycle_engine,
            max_cycles,
        };

        Ok(engine)
    }

    pub fn run(&mut self) -> Result<u64, String> {
        while self
            .max_cycles
            .map_or(true, |max| self.current_cycle() < max)
        {
            self.step()?;
        }
        Ok(self.current_cycle())
    }

    pub fn step(&mut self) -> Result<(), String> {
        self.cycle_engine.run_cycle();
        Ok(())
    }

    pub fn current_cycle(&self) -> u64 {
        self.cycle_engine.current_cycle()
    }
}
