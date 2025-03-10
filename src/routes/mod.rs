//! src/routes/mod.rs

mod health_check;
mod subscriptions;
mod greet;
mod subscriptions_confirm;
mod newsletters;
mod home;
mod login;
mod admin;

pub use health_check::*;
pub use subscriptions::*;
pub use greet::*;
pub use subscriptions_confirm::*;
pub use newsletters::*;
pub use home::*;
pub use login::*;
pub use admin::*;