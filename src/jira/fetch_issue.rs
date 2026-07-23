use rmcp::ErrorData;
use rmcp::Json;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::schemars;
use rmcp::tool;
use rmcp::tool_router;
use schemars::JsonSchema;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::JiraClient;
use super::RmcpToolResult;
use super::key::JiraIssueKey;
use super::model::{JiraIssueOutput, JiraStoryPoints};

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub(super) struct JiraIssueInput {
    key: JiraIssueKey,
    /// Whether to also fetch the optional Story Points value.
    /// Defaults to false, since it lives in an instance-specific custom
    /// field and is not always needed.
    #[serde(default)]
    include_story_points: bool,
}

#[tool_router(router = fetch_issue_tool_router, vis = "pub(super)")]
impl JiraClient {
    #[tool(
        name = "fetch_jira_issue",
        description = "Fetch summary, description, and components of a jira issue. Set include_story_points=true to also fetch the Story Points value",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true
        ),
        execution(task_support = "optional")
    )]
    async fn fetch_issue(
        &self,
        Parameters(JiraIssueInput {
            key,
            include_story_points,
        }): Parameters<JiraIssueInput>,
    ) -> RmcpToolResult<Json<JiraIssueOutput>> {
        let issue = self
            .fetch_issue_from_jira(&key, include_story_points)
            .await?;
        Ok(Json(issue))
    }

    pub async fn fetch_issue_from_jira(
        &self,
        key: &JiraIssueKey,
        include_story_points: bool,
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
        let mut fields_param = "summary,description,components,issuetype".to_string();
        if include_story_points {
            fields_param.push(',');
            fields_param.push_str(&self.story_points_field);
        }
        url.query_pairs_mut().append_pair("fields", &fields_param);

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

        let body = response.json::<serde_json::Value>().await.map_err(|e| {
            ErrorData::internal_error(
                "failed to read Jira issue response",
                Some(json!(e.to_string())),
            )
        })?;
        tracing::debug!("Body:\n{body:?}");

        let story_points = include_story_points
            .then(|| {
                body.get("fields")
                    .and_then(|fields| fields.get(&self.story_points_field))
                    .and_then(|value| value.as_f64())
            })
            .flatten();

        let mut issue: JiraIssueOutput = serde_json::from_value(body).map_err(|e| {
            ErrorData::internal_error(
                "failed to deserialize Jira issue response",
                Some(json!(e.to_string())),
            )
        })?;
        issue.fields.story_points = story_points.map(|v| JiraStoryPoints(v));

        tracing::info!("jira issue fetched: {}", issue.key);
        Ok(issue)
    }
}
