//! src/email_client.rs

use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

#[derive(Clone)]
pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: Client,
    base_url: String,
    authorization_token: Secret<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendEmailRequest<'a> {
    // from: String,
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

impl EmailClient {
    pub fn new(
        base_url: String, 
        sender: SubscriberEmail, 
        authorization_token: Secret<String>, 
        timeout: std::time::Duration
    ) -> Self {
        let http_client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap();

        Self {
            http_client,
            base_url,
            sender,
            authorization_token,
        }
    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        // todo!()
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            // from: self.sender.as_ref().to_owned(),
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject: subject,
            html_body: html_content,
            text_body: text_content,
        };
        let builder = self
            .http_client
            .post(&url)
            .header(
                "Some-sort-of-a-token", 
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claim::{assert_ok, assert_err};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                dbg!(&body);
                body.get("From").is_some() &&
                body.get("To").is_some() &&
                body.get("Subject").is_some() &&
                body.get("HtmlBody").is_some() &&
                body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url, 
            SubscriberEmail::parse(SafeEmail().fake()).unwrap(), 
            Secret::new(Faker.fake()), 
            std::time::Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // todo!()
        let mock_server = MockServer::start().await;
        // let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        // let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(Faker.fake()));
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("Some-sort-of-a-token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = email_client.send_email(subscriber_email, &subject, &content, &content).await;

    }

    #[tokio::test]
    async fn send_email_succeed_if_the_server_returns_200() {
        // todo!()
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let outcome = email_client.send_email(subscriber_email, &subject, &content, &content).await;

        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // todo!()
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let outcome = email_client.send_email(subscriber_email, &subject, &content, &content).await;

        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_timeout() {
        // todo!()
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let outcome = email_client.send_email(subscriber_email, &subject, &content, &content).await;

        assert_err!(outcome);
    }
}