pub mod fifo_buffer;
pub mod fryer;

pub use fifo_buffer::{
    GenericFifoBuffer, BufferItem,
    FriedMeatBuffer, CookedBreadBuffer, AssemblyBuffer
};
pub use fryer::Fryer;