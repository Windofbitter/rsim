use super::connection_manager::ConnectionManager;
use super::cycle_engine::CycleEngine;

pub struct SimulationEngine {
    cycle_engine: CycleEngine,
    max_cycles: Option<u64>,
}

impl SimulationEngine {
    pub fn new(
        connection_manager: ConnectionManager,
        max_cycles: Option<u64>,
    ) -> Result<Self, String> {
        // Create cycle engine and transfer components from connection manager
        let mut cycle_engine = CycleEngine::new();

        // Transfer all components to cycle engine
        for (_id, component) in connection_manager.processing_components {
            cycle_engine.register_processing(component);
        }

        for (_id, component) in connection_manager.memory_components {
            cycle_engine.register_memory(component);
        }

        for (_id, component) in connection_manager.probe_components {
            cycle_engine.register_probe(component);
        }

        // Transfer connections with validation
        for (source, targets) in connection_manager.connections {
            for target in targets {
                cycle_engine
                    .connect(source.clone(), target)
                    .map_err(|e| format!("Connection validation failed: {}", e))?;
            }
        }

        // Transfer memory connections with validation
        for ((proc_id, port), mem_id) in connection_manager.memory_connections {
            cycle_engine
                .connect_memory(proc_id, port, mem_id)
                .map_err(|e| format!("Memory connection validation failed: {}", e))?;
        }

        // Transfer probe connections with validation
        for (source_port, probe_ids) in connection_manager.probes {
            for probe_id in probe_ids {
                cycle_engine
                    .connect_probe(source_port.clone(), probe_id)
                    .map_err(|e| format!("Probe connection validation failed: {}", e))?;
            }
        }

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

    pub fn step(&mut self) -> Result<bool, String> {
        self.cycle_engine.run_cycle();
        Ok(true)
    }

    pub fn current_cycle(&self) -> u64 {
        self.cycle_engine.current_cycle()
    }
}
