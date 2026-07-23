use derive_more::Display;
use rmcp::ErrorData;
use rmcp::Json;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::schemars;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::JiraClient;
use super::RmcpToolResult;
use super::create_subtask::{
    JiraSubtaskAcceptanceCriterion, render_description, validate_description_input,
};
use super::key::JiraIssueKey;

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub(super) struct JiraUpdateIssueDescriptionInput {
    /// Key of the issue to update, e.g. PROJ-123. Must not be a subtask;
    /// the tool refuses to touch any subtask (use
    /// update_jira_subtask_description for those).
    key: JiraIssueKey,
    /// One to two short paragraphs of narrative context: what this issue
    /// delivers, why it's separate, what the current gap is, and what
    /// target state should be true after completion.
    narrative: String,
    /// Given/When/Then scenarios, each independently verifiable. Larger
    /// issues may reasonably need more than a subtask would.
    acceptance_criteria: Vec<JiraSubtaskAcceptanceCriterion>,
    /// Explicitly out-of-scope items. Populate only when the input
    /// explicitly defines exclusions, related work has blurry boundaries,
    /// or a natural extension is deliberately deferred. Omit or leave
    /// empty when there is no useful boundary to call out.
    #[serde(default)]
    out_of_scope: Vec<String>,
}

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[display("Updated Jira issue description: {key}")]
pub struct JiraUpdateIssueDescriptionOutput {
    key: JiraIssueKey,
}

#[tool_router(router = update_issue_description_tool_router, vis = "pub(super)")]
impl JiraClient {
    #[tool(
        name = "update_jira_issue_description",
        description = "Replace the description of an existing Jira issue. Refuses to run on subtasks; use update_jira_subtask_description for those.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true
        ),
        execution(task_support = "optional")
    )]
    async fn update_issue_description(
        &self,
        Parameters(JiraUpdateIssueDescriptionInput {
            key,
            narrative,
            acceptance_criteria,
            out_of_scope,
        }): Parameters<JiraUpdateIssueDescriptionInput>,
    ) -> RmcpToolResult<Json<JiraUpdateIssueDescriptionOutput>> {
        let issue = self
            .update_issue_description_in_jira(&key, &narrative, &acceptance_criteria, &out_of_scope)
            .await?;
        Ok(Json(issue))
    }

    pub async fn update_issue_description_in_jira(
        &self,
        key: &JiraIssueKey,
        narrative: &str,
        acceptance_criteria: &[JiraSubtaskAcceptanceCriterion],
        out_of_scope: &[String],
    ) -> RmcpToolResult<JiraUpdateIssueDescriptionOutput> {
        if !key.is_allowed(&self.allowed_projects) {
            return Err(ErrorData::invalid_params(
                format!("Jira issue {key} is not allowed"),
                None,
            ));
        }

        validate_description_input(narrative, acceptance_criteria)?;

        let issue = self.fetch_issue_from_jira(key, false).await?;
        if issue.fields.issuetype.subtask {
            return Err(ErrorData::invalid_params(
                format!(
                    "Jira issue {key} is a subtask; this tool refuses to touch subtasks, use update_jira_subtask_description instead"
                ),
                None,
            ));
        }

        let description = render_description(narrative, acceptance_criteria, out_of_scope);

        tracing::debug!("update jira issue description: {key}");
        let url = self
            .base_url
            .join("rest/api/2/issue/")
            .and_then(|url| url.join(&key.to_string()))
            .map_err(|e| {
                ErrorData::internal_error(
                    "failed to construct Jira update issue URL",
                    Some(json!(e.to_string())),
                )
            })?;

        let body = json!({
            "fields": {
                "description": description,
            }
        });

        let mut request = self
            .http
            .put(url)
            .header("Accept", "application/json")
            .json(&body);

        if let Some(api_token) = &self.api_token {
            request = request.bearer_auth(api_token.expose_secret());
        }

        let response = request.send().await.map_err(|e| {
            ErrorData::internal_error("Jira HTTP request failed", Some(json!(e.to_string())))
        })?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(ErrorData::internal_error(
                format!("Jira returned non-success status {status}"),
                Some(json!(body_text)),
            ));
        }

        tracing::info!("jira issue description updated: {key}");
        Ok(JiraUpdateIssueDescriptionOutput {
            key: key.to_owned(),
        })
    }
}
