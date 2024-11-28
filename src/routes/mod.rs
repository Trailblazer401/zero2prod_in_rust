//! src/routes/mod.rs

mod health_check;
mod subscriptions;
mod greet;
mod subscriptions_confirm;

pub use health_check::*;
pub use subscriptions::*;
pub use greet::*;
pub use subscriptions_confirm::*;