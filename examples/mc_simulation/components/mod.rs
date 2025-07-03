//! McDonald's simulation components
//! 
//! This module contains all the components needed for the McDonald's production line simulation,
//! including FIFO buffers, producers, managers, and consumers.

pub mod fifo;
pub mod baker;
pub mod fryer;
pub mod assembler;
pub mod bread_manager;
pub mod meat_manager;
pub mod assembler_manager;

pub use fifo::FIFOData;
pub use baker::Baker;
pub use fryer::Fryer;
pub use assembler::Assembler;
pub use bread_manager::BreadManager;
pub use meat_manager::MeatManager;
pub use assembler_manager::AssemblerManager;