//! src/routes/admin/password/mod.rs

mod get;
mod post;

pub use get::change_password_form;
pub use post::change_password;