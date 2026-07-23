use anyhow::{Error, Result, anyhow};
use derive_more::{Deref, Display};
use regex::Regex;
use reqwest::{Client, Url};
use rmcp::ErrorData;
use rmcp::Json;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::schemars;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;

type RmcpToolResult<T> = std::result::Result<T, ErrorData>;

#[derive(Debug, Clone, Deref, Deserialize, Display, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct JiraIssueProject(String);

#[derive(Debug, Clone, Deserialize, Display, JsonSchema, Serialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
#[display("{project}-{id}")]
pub struct JiraIssueKey {
    project: JiraIssueProject,
    id: u32,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
struct JiraIssueInput {
    key: JiraIssueKey,
}

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[display("Jira issue: {key}\n{fields}")]
pub struct JiraIssueOutput {
    key: JiraIssueKey,
    fields: JiraIssueFields,
}

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[display("Summary: {summary}\nDescription:\n{description}")]
struct JiraIssueFields {
    summary: String,
    description: String,
}

#[derive(Clone, Debug)]
pub struct JiraClient {
    http: Client,
    base_url: Url,
    api_token: Option<SecretString>,
    allowed_projects: Vec<JiraIssueProject>,
}

#[derive(Debug)]
pub struct JiraClientBuilder {
    base_url: Url,
    api_token: Option<SecretString>,
    allowed_projects: Vec<JiraIssueProject>,
}

impl From<JiraIssueKey> for String {
    fn from(key: JiraIssueKey) -> String {
        key.to_string()
    }
}

impl TryFrom<String> for JiraIssueKey {
    type Error = Error;
    fn try_from(key: String) -> Result<Self, Self::Error> {
        key.parse()
    }
}

impl FromStr for JiraIssueKey {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^(?<proj>[A-Z][A-Z0-9]+)-(?<id>[1-9][0-9]*)$")?;
        if let Some(caps) = re.captures(s) {
            let project = JiraIssueProject(caps["proj"].to_owned());
            let id = caps["id"].parse().unwrap();
            Ok(Self { project, id })
        } else {
            Err(anyhow!("expected format like PROJ-123"))
        }
    }
}

impl FromStr for JiraIssueProject {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^[A-Z][A-Z0-9]+$")?;
        if re.is_match(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(anyhow!("expected format like PROJ"))
        }
    }
}

impl JiraIssueKey {
    fn is_allowed(&self, allowed: &[JiraIssueProject]) -> bool {
        allowed.iter().any(|allowed| &self.project == allowed)
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
        execution(task_support = "optional")
    )]
    async fn fetch_issue(
        &self,
        Parameters(JiraIssueInput { key }): Parameters<JiraIssueInput>,
    ) -> RmcpToolResult<Json<JiraIssueOutput>> {
        let issue = self.fetch_issue_from_jira(&key).await?;
        Ok(Json(issue))
    }

    pub async fn fetch_issue_from_jira(
        &self,
        key: &JiraIssueKey,
    ) -> RmcpToolResult<JiraIssueOutput> {
        if !key.is_allowed(&self.allowed_projects) {
            return Err(ErrorData::invalid_params(
                format!("Jira issue {key} is not allowed"),
                None,
            ));
        }

        tracing::debug!("fetch jira issue: {key}");
        let mut url = self
            .base_url
            .join("rest/api/2/issue/")
            .and_then(|url| url.join(&key.to_string()))
            .map_err(|e| {
                ErrorData::internal_error(
                    "failed to construct Jira issue URL",
                    Some(json!(e.to_string())),
                )
            })?;
        url.query_pairs_mut()
            .append_pair("fields", "summary,description");

        let mut request = self.http.get(url).header("Accept", "application/json");

        if let Some(api_token) = &self.api_token {
            request = request.bearer_auth(api_token.expose_secret());
        }

        let response = request.send().await.map_err(|e| {
            ErrorData::internal_error("Jira HTTP request failed", Some(json!(e.to_string())))
        })?;

        let status = response.status();
        if !status.is_success() {
            return Err(ErrorData::internal_error(
                format!("Jira returned non-success status {status}"),
                None,
            ));
        }

        let issue = response.json::<JiraIssueOutput>().await.map_err(|e| {
            ErrorData::internal_error(
                "failed to deserialize Jira issue response",
                Some(json!(e.to_string())),
            )
        })?;
        tracing::info!("jira issue fetched: {}", issue.key);
        Ok(issue)
    }
}

impl Default for JiraClientBuilder {
    fn default() -> Self {
        let base_url = "http://localhost:8080".parse().unwrap();
        Self {
            base_url,
            api_token: None,
            allowed_projects: Vec::new(),
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

    pub fn with_allowed_projects(
        mut self,
        allowed_projects: impl Into<Vec<JiraIssueProject>>,
    ) -> Self {
        self.allowed_projects = allowed_projects.into();
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
            allowed_projects: self.allowed_projects,
        }
    }
}
