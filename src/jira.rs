use anyhow::{anyhow, Context, Error, Result};
use derive_more::{Deref, Display};
use regex::Regex;
use reqwest::{Client, Url};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone, Deref, Deserialize, Display)]
#[serde(transparent)]
pub struct JiraIssueKey(String);

#[derive(Debug)]
pub struct JiraClient {
    base_url: Url,
    api_token: Option<SecretString>,
    http: Client,
}

#[derive(Debug)]
pub struct JiraClientBuilder {
    base_url: Url,
    api_token: Option<SecretString>,
}

#[derive(Debug, Deserialize)]
pub struct JiraIssueResponse {
    pub key: JiraIssueKey,
    pub fields: JiraIssueResponseFields,
}

#[derive(Debug, Deserialize)]
pub struct JiraIssueResponseFields {
    pub summary: String,
    pub description: String,
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

impl JiraClient {
    pub fn builder() -> JiraClientBuilder {
        JiraClientBuilder::default()
    }

    pub async fn get_issue(&self, key: &JiraIssueKey) -> Result<JiraIssueResponse> {
        let mut url = self.base_url
            .join("rest/api/2/issue/")
            .and_then(|url| url.join(&key))
            .context("failed to construct Jira issue URL")?;
        url.query_pairs_mut()
            .append_pair("fields", "summary,description");

        let mut request = self.http
            .get(url)
            .header("Accept", "application/json");

        if let Some(api_token) = &self.api_token {
            request = request.bearer_auth(api_token.expose_secret());
        }

        let response = request.send().await
            .context("Jira HTTP request failed")?;

        let status = response.status();
        if !status.is_success() {
            return Err(anyhow!("Jira returned non-success status {status}"));
        }

        response
            .json::<JiraIssueResponse>()
            .await
            .context("failed to deserialize Jira issue response")
    }
}

impl Default for JiraClientBuilder {
    fn default() -> Self {
        let base_url = "http://localhost:8080".parse().unwrap();
        Self {
            base_url,
            api_token: None,
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

    pub fn build(self) -> JiraClient {
        if self.api_token.is_none() {
            tracing::warn!("no API token configured");
        }

        let http = Client::new();
        JiraClient {
            base_url: self.base_url,
            api_token: self.api_token,
            http,
        }
    }
}
