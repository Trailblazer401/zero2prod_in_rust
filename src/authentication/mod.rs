//! src/authentication/mod.rs

mod middleware;
mod password;

pub use password::{
    change_password, validate_credentials, 
    AuthError, Credentails,
};

pub use middleware::{UserId, reject_anonymous_users};