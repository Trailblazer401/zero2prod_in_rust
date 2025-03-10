//! src/routes/login/get.rs

use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use std::fmt::Write;

pub async fn login_form(
    flash_messages: IncomingFlashMessages,
) -> HttpResponse {
    // let error_html = match request.cookie("_flash") {
    //     None => "".into(),
    //     Some(cookie) => {
    //         format!("<p><i>{}</i></p>", cookie.value())
    //     }
    // };
    let mut error_msg = String::new();
    for m in flash_messages.iter().filter(|m| m.level() == Level::Error || m.level() == Level::Info) {
        writeln!(error_msg, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta http-equiv="content-type" content="text/html"; charset="UTF-8">
                <title>Login</title>
            </head>
            <body>
                {error_msg}
                <form action="/login" method="POST">
                    <label>Username
                        <input type="text" placeholder="Enter username" name="username" required>
                    </label>
                    
                    <label>Password
                        <input type="password" placeholder="Enter password" name="password" required>
                    </label>
                    <button type="submit">Login</button>
                </form>
            </body>
        </html>
        "#,);

    HttpResponse::Ok()
        .content_type(ContentType::html())
        // .cookie(
        //     Cookie::build("_flash", "")
        //         .max_age(Duration::ZERO)
        //         .finish(),
        // )
        .body(html)

}