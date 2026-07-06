use anyhow::{anyhow, Error, Result};
use derive_more::{Deref, Display};
use regex::Regex;
use reqwest::{Client, Url};
use rmcp::Json;
use rmcp::ErrorData;
use rmcp::schemars;
use rmcp::{tool, tool_router};
use rmcp::handler::server::wrapper::Parameters;
use schemars::JsonSchema;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;

type RmcpToolResult<T> = std::result::Result<T, ErrorData>;

#[derive(Debug, Clone, Deref, Deserialize, Display)]
#[serde(transparent)]
pub struct JiraIssueKeyPrefix(String);

#[derive(Debug, Clone, Deref, Deserialize, Display, JsonSchema, Serialize)]
#[serde(transparent)]
pub struct JiraIssueKey(String);

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct JiraIssueInput {
    key: JiraIssueKey,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct JiraIssueOutput {
    key: JiraIssueKey,
    fields: JiraIssueFields,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
struct JiraIssueFields {
    summary: String,
    description: String,
}

#[derive(Clone, Debug)]
pub struct JiraClient {
    http: Client,
    base_url: Url,
    api_token: Option<SecretString>,
    allowed_key_prefixes: Vec<JiraIssueKeyPrefix>,
}

#[derive(Debug)]
pub struct JiraClientBuilder {
    base_url: Url,
    api_token: Option<SecretString>,
    allowed_key_prefixes: Vec<JiraIssueKeyPrefix>,
}

impl FromStr for JiraIssueKey {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^[A-Z][A-Z0-9]+-[1-9][0-9]*$")?;
        if re.is_match(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(anyhow!("expected format like PROJ-123"))
        }
    }
}

impl FromStr for JiraIssueKeyPrefix {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^[A-Z][A-Z0-9]+$")?;
        if re.is_match(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(anyhow!("expected format like PROJ-123"))
        }
    }
}

impl JiraIssueKey {
    pub fn is_allowed(&self, allowed: &[JiraIssueKeyPrefix]) -> bool {
        allowed
            .iter()
            .any(|prefix| self.starts_with(prefix.as_str()))
    }
}

impl JiraClient {
    pub fn builder() -> JiraClientBuilder {
        JiraClientBuilder::default()
    }
}

#[tool_router(server_handler)]
impl JiraClient {
    #[tool(
        name = "fetch_jira_issue",
        description = "Fetch summary and description of a jira issue",
        annotations(read_only_hint = true),
        execution(task_support = "optional"),
    )]
    async fn fetch_issue(&self, Parameters(JiraIssueInput { key }): Parameters<JiraIssueInput>) -> RmcpToolResult<Json<JiraIssueOutput>> {
        if !key.is_allowed(&self.allowed_key_prefixes) {
            return Err(ErrorData::invalid_params(
                format!("Jira issue {key} is not allowed"),
                None,
            ));
        }

        tracing::debug!("fetch jira issue: {}", key);
        let mut url = self
            .base_url
            .join("rest/api/2/issue/")
            .and_then(|url| url.join(&key))
            .map_err(|e| ErrorData::internal_error(
                "failed to construct Jira issue URL",
                Some(json!(e.to_string()))
            ))?;
        url.query_pairs_mut()
            .append_pair("fields", "summary,description");

        let mut request = self.http
            .get(url)
            .header("Accept", "application/json");

        if let Some(api_token) = &self.api_token {
            request = request.bearer_auth(api_token.expose_secret());
        }

        let response = request.send().await
            .map_err(|e| ErrorData::internal_error(
                "Jira HTTP request failed",
                Some(json!(e.to_string()))
            ))?;

        let status = response.status();
        if !status.is_success() {
            return Err(ErrorData::internal_error(
                format!("Jira returned non-success status {status}"),
                None,
            ));
        }

        let issue = response
            .json::<JiraIssueOutput>()
            .await
            .map_err(|e| ErrorData::internal_error(
                "failed to deserialize Jira issue response",
                Some(json!(e.to_string()))
            ))?;
        tracing::info!("jira issue fetched: {}", issue.key);
        Ok(Json(issue))
    }
}

impl Default for JiraClientBuilder {
    fn default() -> Self {
        let base_url = "http://localhost:8080".parse().unwrap();
        Self {
            base_url,
            api_token: None,
            allowed_key_prefixes: Vec::new(),
        }
    }
}

impl JiraClientBuilder {
    pub fn with_base_url(mut self, base_url: Url) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn with_api_token(mut self, api_token: SecretString) -> Self {
        self.api_token = Some(api_token);
        self
    }

    pub fn with_allowed_key_prefixes(mut self, allowed_key_prefixes: impl Into<Vec<JiraIssueKeyPrefix>>) -> Self {
        self.allowed_key_prefixes = allowed_key_prefixes.into();
        self
    }

    pub fn build(self) -> JiraClient {
        if self.api_token.is_none() {
            tracing::warn!("no API token configured");
        }
        
        let http = Client::new();
        JiraClient {
            http,
            base_url: self.base_url,
            api_token: self.api_token,
            allowed_key_prefixes: self.allowed_key_prefixes,
        }
    }
}
