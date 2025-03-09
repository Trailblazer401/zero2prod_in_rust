//! src/routes/admin/mod.rs

mod dashboard;
mod password;
mod logout;
mod newsletter;

pub use dashboard::admin_dashboard;
pub use password::*;
pub use logout::*;
pub use newsletter::*;