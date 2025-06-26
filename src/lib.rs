#![no_std]
#![allow(async_fn_in_trait)]

pub mod common;
pub mod registers;
pub mod heartbeat;

#[cfg(feature = "blocking")]
pub mod blocking;

#[cfg(feature = "async")]
pub mod async_i2c;

// Re-export common types at crate root
pub use common::{DRV2605L_ADDR, Effect, Error, Library, Mode, MotorType};
pub use heartbeat::HeartbeatPattern;

// Re-export the appropriate driver based on features
#[cfg(all(feature = "blocking", not(feature = "async")))]
pub use blocking::Drv2605l;

#[cfg(feature = "async")]
pub use async_i2c::Drv2605l;

// If both features are enabled, require explicit module usage
#[cfg(all(feature = "blocking", feature = "async"))]
pub mod prelude {
    pub use crate::blocking::Drv2605l as BlockingDrv2605l;
    pub use crate::async_i2c::Drv2605l as AsyncDrv2605l;
}