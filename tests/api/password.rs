//! tests/api/password.rs

use crate::helpers::{spawn_app, assert_is_redirect_to};
use uuid::Uuid;

#[tokio::test]
async fn you_must_be_logged_in_to_see_change_password_form() {
    let app = spawn_app().await;
    let reponse = app.get_change_password().await;

    assert_is_redirect_to(&reponse, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_change_password() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    let reponse = app.post_change_password(
        &serde_json::json!({
            "current_password": Uuid::new_v4().to_string(),
            "new_password": &new_password,
            "new_password_check": &new_password,
        })
    ).await;

    assert_is_redirect_to(&reponse, "/login");
}

#[tokio::test]
async fn new_password_fields_must_match() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let another_password = Uuid::new_v4().to_string();

    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    })).await;

    let response = app.post_change_password(&serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": &new_password,
        "new_password_check": &another_password,
    })).await;

    assert_is_redirect_to(&response, "/admin/password");

    let html = app.get_change_password_html().await;
    assert!(html.contains(
        "<p><i>You entered two different new passwords - \
        the field values must match.</i></p>"
    ));
}

#[tokio::test]
async fn password_must_be_verified_before_changing() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let wrong_password = Uuid::new_v4().to_string();

    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
        })).await;

    let response = app.post_change_password(&serde_json::json!({
        "current_password": &wrong_password,
        "new_password": &new_password,
        "new_password_check": &new_password
    })).await;

    assert_is_redirect_to(&response, "/admin/password");

    let html = app.get_change_password_html().await;
    assert!(html.contains(
        "<p><i>The current password is incorrect!</i></p>"
    ));
}

#[tokio::test]
async fn new_password_is_too_short_or_too_long() {
    let app = spawn_app().await;
    let short_password = "too_short";
    let long_password = "too_looooooooooooooooooo\
        ooooooooooooooooooooooooooooooooooooooooooooooooo\
        oooooooooooooooooooooooooooooooooooooooooooooooooo\
        oooooooooooooooooooooooooooooooooooooooooooooooong";
    // let wrong_password = Uuid::new_v4().to_string();

    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
        })).await;

    let change_body = |password| {serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": &password,
        "new_password_check": &password
    })};

    let response = app.post_change_password(&change_body(short_password)).await;

    assert_is_redirect_to(&response, "/admin/password");

    let html = app.get_change_password_html().await;
    assert!(html.contains(
        "<p><i>The new password doesn't meet the OWASP requirement.</i></p>"
    ));

    app.post_change_password(&change_body(long_password)).await;
    assert!(app.get_change_password_html().await.contains(
        "<p><i>The new password doesn't meet the OWASP requirement.</i></p>"
    ));
}

#[tokio::test]
async fn logout_clears_session_state() {
    let app = spawn_app().await;

    let login_body = &serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
        });
    let response = app.post_login(&login_body).await;

    assert_is_redirect_to(&response, "/admin/dashboard");

    let html = app.get_admin_dashboard_html().await;
    assert!(html.contains(&format!("Welcome {}", app.test_user.username)));

    let response = app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    let html = app.get_login_html().await;
    assert!(html.contains(r#"<p><i>You have successfully logged out.</i></p>"#));

    let response = app.get_admin_dashboard().await;
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn change_password_func_work() {
    let app = spawn_app().await;

    let new_password = Uuid::new_v4().to_string();

    let login_body = &serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
        });
    let response = app.post_login(&login_body).await;

    assert_is_redirect_to(&response, "/admin/dashboard");

    let response = app.post_change_password(&serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": &new_password,
        "new_password_check": &new_password
    })).await;
    assert_is_redirect_to(&response, "/admin/password");

    let html = app.get_change_password_html().await;
    assert!(html.contains(&format!("<p><i>Your password has been changed successfully.</i></p>",)));

    let login_body = &serde_json::json!({
        "username": &app.test_user.username,
        "password": &new_password
        });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

}