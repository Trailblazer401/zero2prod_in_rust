//! src/routes/login/get.rs

use actix_web::{web, HttpResponse, http::header::ContentType};
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;

use crate::startup::HmacSecret;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));

        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;

        Ok(self.error)
    }
}

pub async fn login_form(
    query: Option<web::Query<QueryParams>>,
    secret: web::Data<HmacSecret>,
) -> HttpResponse {
    let error_html = match query {
        None => "".into(),
        // Some(query) => format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&query.0.error)),
        Some(query) => match query.0.verify(&secret) {
            Ok(error) => format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&error)),
            Err(e) => {
                tracing::warn!(
                    error.message = %e,
                    error.cause_chain = ?e,
                    "Failed to verify the query parameters using HMAC tag."
                );
                "".into()
            }
        },

    };

//     let html = format!(
//         r#"
//         <!DOCTYPE html>
//         <html lang="en">
//             <head>
//                 <meta charset="UTF-8">
//                 <meta name="viewport" content="width=device-width, initial-scale=1.0">
//                 <meta http-equiv="content-type" content="text/html"; charset="UTF-8">
//                 <title>Login</title>
//             </head>
//             <body>
//                 {error_html}
//                 <form action="/login" method="POST">
//                     <label>Username
//                         <input type="text" placeholder="Enter username" name="username" required>
//                     </label>
                    
//                     <label>Password
//                         <input type="password" placeholder="Enter password" name="password" required>
//                     </label>
//                     <button type="submit">Login</button>
//                 </form>
//             </body>
//         </html>
//         "#,
// );

    HttpResponse::Ok()
        .content_type(ContentType::html())
        // .body(html)
        .body(format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <meta http-equiv="content-type" content="text/html"; charset="UTF-8">
                    <title>Login</title>
                </head>
                <body>
                    {error_html}
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
            "#,
        ))
}