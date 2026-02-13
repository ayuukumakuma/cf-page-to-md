use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::cli::CSS_REJECT_REQUEST_PATTERN;
use crate::error::AppError;

const DEFAULT_BASE_URL: &str = "https://api.cloudflare.com/client/v4";

#[derive(Debug, Clone)]
pub struct MarkdownClient {
    http: Client,
    endpoint: String,
    api_token: String,
}

impl MarkdownClient {
    pub fn new(account_id: &str, api_token: &str) -> Result<Self, AppError> {
        Self::new_with_base_url(account_id, api_token, DEFAULT_BASE_URL)
    }

    pub fn new_with_base_url(
        account_id: &str,
        api_token: &str,
        base_url: &str,
    ) -> Result<Self, AppError> {
        let trimmed = base_url.trim_end_matches('/');
        let endpoint = format!("{trimmed}/accounts/{account_id}/browser-rendering/markdown");

        Ok(Self {
            http: Client::builder().build()?,
            endpoint,
            api_token: api_token.to_string(),
        })
    }

    pub async fn fetch_markdown(&self, url: &str) -> Result<String, AppError> {
        let body = build_markdown_request(url);

        let response = self
            .http
            .post(&self.endpoint)
            .bearer_auth(&self.api_token)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let raw_body = response.text().await?;

        let envelope: MarkdownResponseEnvelope =
            serde_json::from_str(&raw_body).map_err(|err| {
                AppError::InvalidResponse(format!(
                    "JSON解析に失敗しました: {err}; body={}",
                    truncate_for_error(&raw_body)
                ))
            })?;

        if !status.is_success() || !envelope.success {
            let message = envelope
                .error_message()
                .unwrap_or_else(|| "Cloudflare APIがエラーを返しました".to_string());

            return Err(AppError::Api {
                status: Some(status.as_u16()),
                message,
            });
        }

        let markdown = envelope
            .result
            .ok_or_else(|| AppError::InvalidResponse("`result` が存在しません".to_string()))?
            .into_markdown();

        Ok(markdown)
    }
}

fn truncate_for_error(text: &str) -> String {
    const LIMIT: usize = 300;

    if text.len() <= LIMIT {
        return text.to_string();
    }

    let mut s = text.chars().take(LIMIT).collect::<String>();
    s.push_str("...");
    s
}

#[derive(Debug, Serialize)]
pub struct MarkdownRequest<'a> {
    pub url: &'a str,
    #[serde(rename = "rejectRequestPattern")]
    pub reject_request_pattern: Vec<&'a str>,
}

pub fn build_markdown_request(url: &str) -> MarkdownRequest<'_> {
    MarkdownRequest {
        url,
        reject_request_pattern: vec![CSS_REJECT_REQUEST_PATTERN],
    }
}

#[derive(Debug, Deserialize)]
struct MarkdownResponseEnvelope {
    success: bool,
    errors: Option<Vec<CloudflareError>>,
    result: Option<MarkdownResult>,
}

impl MarkdownResponseEnvelope {
    fn error_message(&self) -> Option<String> {
        self.errors
            .as_ref()
            .filter(|errors| !errors.is_empty())
            .map(|errors| {
                errors
                    .iter()
                    .map(|e| match e.code {
                        Some(code) => format!("[{code}] {}", e.message),
                        None => e.message.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            })
    }
}

#[derive(Debug, Deserialize)]
struct CloudflareError {
    code: Option<i64>,
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum MarkdownResult {
    Text(String),
    Object { markdown: String },
}

impl MarkdownResult {
    fn into_markdown(self) -> String {
        match self {
            Self::Text(text) => text,
            Self::Object { markdown } => markdown,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::error::AppError;

    use super::{build_markdown_request, MarkdownClient};

    #[test]
    fn request_body_always_contains_css_reject_pattern() {
        let request = build_markdown_request("https://example.com");
        let value = serde_json::to_value(request).expect("serialize failed");

        assert_eq!(
            value["url"],
            Value::String("https://example.com".to_string())
        );
        assert_eq!(value["rejectRequestPattern"], json!(["/^.*\\.(css)/"]));
    }

    #[tokio::test]
    async fn fetch_markdown_success_with_string_result() {
        let server = MockServer::start().await;
        let response = ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "result": "# Example"
        }));

        Mock::given(method("POST"))
            .and(path(
                "/client/v4/accounts/test-account/browser-rendering/markdown",
            ))
            .and(header("authorization", "Bearer test-token"))
            .and(body_json(json!({
                "url": "https://example.com",
                "rejectRequestPattern": ["/^.*\\.(css)/"]
            })))
            .respond_with(response)
            .mount(&server)
            .await;

        let base_url = format!("{}/client/v4", server.uri());
        let client = MarkdownClient::new_with_base_url("test-account", "test-token", &base_url)
            .expect("client init failed");

        let markdown = client
            .fetch_markdown("https://example.com")
            .await
            .expect("fetch failed");

        assert_eq!(markdown, "# Example");
    }

    #[tokio::test]
    async fn fetch_markdown_supports_object_result() {
        let server = MockServer::start().await;
        let response = ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "result": {
                "markdown": "# Object Result"
            }
        }));

        Mock::given(method("POST"))
            .and(path(
                "/client/v4/accounts/test-account/browser-rendering/markdown",
            ))
            .respond_with(response)
            .mount(&server)
            .await;

        let base_url = format!("{}/client/v4", server.uri());
        let client = MarkdownClient::new_with_base_url("test-account", "test-token", &base_url)
            .expect("client init failed");

        let markdown = client
            .fetch_markdown("https://example.com")
            .await
            .expect("fetch failed");

        assert_eq!(markdown, "# Object Result");
    }

    #[tokio::test]
    async fn fetch_markdown_returns_api_error() {
        let server = MockServer::start().await;
        let response = ResponseTemplate::new(400).set_body_json(json!({
            "success": false,
            "errors": [{"code": 1001, "message": "invalid request"}]
        }));

        Mock::given(method("POST"))
            .and(path(
                "/client/v4/accounts/test-account/browser-rendering/markdown",
            ))
            .respond_with(response)
            .mount(&server)
            .await;

        let base_url = format!("{}/client/v4", server.uri());
        let client = MarkdownClient::new_with_base_url("test-account", "test-token", &base_url)
            .expect("client init failed");

        let error = client
            .fetch_markdown("https://example.com")
            .await
            .expect_err("should fail");

        match error {
            AppError::Api { status, message } => {
                assert_eq!(status, Some(400));
                assert!(message.contains("invalid request"));
            }
            _ => panic!("unexpected error: {error:?}"),
        }
    }
}
