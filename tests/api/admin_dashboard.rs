//! test/api/admin_dashboard.rs

use crate::helpers::{spawn_app, assert_is_redirect_to};

#[tokio::test]
async fn user_must_be_logged_in_to_access_the_admin_dashboard() {
    let app = spawn_app().await;

    let reponse = app.get_admin_dashboard().await;

    assert_is_redirect_to(&reponse, "/login");
}