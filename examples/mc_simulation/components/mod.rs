//! McDonald's simulation components
//! 
//! This module contains all the components needed for the McDonald's production line simulation,
//! including FIFO buffers, producers, managers, and consumers.

pub mod fifo;

pub use fifo::{FIFOData, FIFOModule};