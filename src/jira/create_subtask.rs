use derive_more::Display;
use rmcp::ErrorData;
use rmcp::Json;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{PromptMessage, Role};
use rmcp::schemars;
use rmcp::{prompt, prompt_router, tool, tool_router};
use schemars::JsonSchema;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::JiraClient;
use super::RmcpToolResult;
use super::key::JiraIssueKey;
use anyhow::Error;

/// Style guidelines for writing Jira subtask summaries and descriptions.
/// Kept out of `create_jira_subtask`'s tool description so it is only
/// loaded into the LLM's context when actually needed, not on every
/// tool listing.
const SUBTASK_WRITING_GUIDELINES: &str = include_str!("../../resources/create-jira-subtasks.md");

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub(super) struct CreateSubtaskPromptInput {
    /// Key of the parent issue the subtask(s) will be created under, e.g. PROJ-123.
    parent: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub(super) struct JiraCreateSubtaskInput {
    /// Key of the parent issue the subtask is created under, e.g. PROJ-123.
    /// The parent issue must not itself be a subtask.
    parent: JiraIssueKey,
    /// Summary of the new subtask. Imperative mood, action-oriented, ~60
    /// characters max, no Jira ticket prefix, specific enough to
    /// distinguish it from sibling subtasks.
    summary: String,
    /// One to two short paragraphs of narrative context: what this subtask
    /// delivers, why it's separate, what the current gap is, and what
    /// target state should be true after completion.
    narrative: String,
    /// 1-3 Given/When/Then scenarios, each independently verifiable.
    acceptance_criteria: Vec<JiraSubtaskAcceptanceCriterion>,
    /// Explicitly out-of-scope items. Populate only when the input
    /// explicitly defines exclusions, related work has blurry boundaries,
    /// or a natural extension is deliberately deferred. Omit or leave
    /// empty when there is no useful boundary to call out.
    #[serde(default)]
    out_of_scope: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct JiraSubtaskAcceptanceCriterion {
    /// Short, concrete scenario name, e.g. "Empty description is rejected".
    scenario: String,
    /// Free-text Given/When/Then steps for this scenario, e.g.
    /// "Given ...\nWhen ...\nThen ...". Free-form: omit Given when there's
    /// no meaningful precondition, chain steps with "And", or express a
    /// Scenario Outline with Examples if useful.
    steps: String,
}

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[display("Created Jira subtask: {key}")]
pub struct JiraCreateSubtaskOutput {
    key: JiraIssueKey,
}

#[derive(Debug, Deserialize)]
struct JiraCreatedIssueResponse {
    key: String,
}

/// Render the structured subtask description fields into Jira wiki markup.
/// Keeping this deterministic on the server removes an entire class of
/// LLM markup mistakes (wrong heading, malformed code blocks, stray
/// "Feature:"/"Scenario:" keywords).
pub(super) fn render_description(
    narrative: &str,
    acceptance_criteria: &[JiraSubtaskAcceptanceCriterion],
    out_of_scope: &[String],
) -> String {
    let mut out = narrative.trim().to_string();

    out.push_str("\n\nh2. Acceptance Criteria\n");
    for ac in acceptance_criteria {
        out.push_str(&format!(
            "\n{{code:title={}}}\n{}\n{{code}}\n",
            ac.scenario.trim(),
            ac.steps.trim(),
        ));
    }

    if !out_of_scope.is_empty() {
        out.push_str("\nh2. Out of Scope\n");
        for item in out_of_scope {
            out.push_str(&format!("* {}\n", item.trim()));
        }
    }

    out
}

#[tool_router(router = create_subtask_tool_router, vis = "pub(super)")]
impl JiraClient {
    #[tool(
        name = "create_jira_subtask",
        description = "Create a subtask under a parent Jira issue. Use the create_jira_subtasks prompt first to load the summary/narrative/acceptance-criteria style guidelines.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false
        ),
        execution(task_support = "optional")
    )]
    async fn create_subtask(
        &self,
        Parameters(JiraCreateSubtaskInput {
            parent,
            summary,
            narrative,
            acceptance_criteria,
            out_of_scope,
        }): Parameters<JiraCreateSubtaskInput>,
    ) -> RmcpToolResult<Json<JiraCreateSubtaskOutput>> {
        let issue = self
            .create_subtask_in_jira(
                &parent,
                &summary,
                &narrative,
                &acceptance_criteria,
                &out_of_scope,
            )
            .await?;
        Ok(Json(issue))
    }

    pub async fn create_subtask_in_jira(
        &self,
        parent: &JiraIssueKey,
        summary: &str,
        narrative: &str,
        acceptance_criteria: &[JiraSubtaskAcceptanceCriterion],
        out_of_scope: &[String],
    ) -> RmcpToolResult<JiraCreateSubtaskOutput> {
        if !parent.is_allowed(&self.allowed_projects) {
            return Err(ErrorData::invalid_params(
                format!("Jira issue {parent} is not allowed"),
                None,
            ));
        }

        if narrative.trim().is_empty() {
            return Err(ErrorData::invalid_params(
                "Jira subtask narrative must not be empty",
                None,
            ));
        }

        if acceptance_criteria.is_empty() || acceptance_criteria.len() > 3 {
            return Err(ErrorData::invalid_params(
                "Jira subtask must have between 1 and 3 acceptance criteria",
                None,
            ));
        }

        for ac in acceptance_criteria {
            if ac.scenario.trim().is_empty() || ac.steps.trim().is_empty() {
                return Err(ErrorData::invalid_params(
                    "Jira subtask acceptance criteria must have non-empty scenario and steps",
                    None,
                ));
            }
        }

        let description = render_description(narrative, acceptance_criteria, out_of_scope);

        let parent_fields = self.fetch_issue_from_jira(parent, false).await?;
        if parent_fields.fields.issuetype.subtask {
            return Err(ErrorData::invalid_params(
                format!(
                    "Jira issue {parent} is itself a subtask ({}); subtasks cannot have subtasks",
                    parent_fields.fields.issuetype.name
                ),
                None,
            ));
        }

        if self
            .non_subtaskable_issuetypes
            .iter()
            .any(|denied| denied.eq_ignore_ascii_case(&parent_fields.fields.issuetype.name))
        {
            return Err(ErrorData::invalid_params(
                format!(
                    "Jira issue {parent} is a {}; this issue type must not receive subtasks",
                    parent_fields.fields.issuetype.name
                ),
                None,
            ));
        }

        tracing::debug!("create jira subtask under: {parent}");
        let url = self.base_url.join("rest/api/2/issue").map_err(|e| {
            ErrorData::internal_error(
                "failed to construct Jira create issue URL",
                Some(json!(e.to_string())),
            )
        })?;

        let body = json!({
            "fields": {
                "project": { "key": parent.project.to_string() },
                "parent": { "key": parent.to_string() },
                "issuetype": { "name": self.subtask_issuetype },
                "summary": summary,
                "description": description,
            }
        });

        let mut request = self
            .http
            .post(url)
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

        let created: JiraCreatedIssueResponse = response.json().await.map_err(|e| {
            ErrorData::internal_error(
                "failed to read Jira create issue response",
                Some(json!(e.to_string())),
            )
        })?;

        let key: JiraIssueKey = created.key.parse().map_err(|e: Error| {
            ErrorData::internal_error(
                "failed to parse key of created Jira subtask",
                Some(json!(e.to_string())),
            )
        })?;

        tracing::info!("jira subtask created: {key} under {parent}");
        Ok(JiraCreateSubtaskOutput { key })
    }
}

#[prompt_router(router = "create_subtask_prompt_router", vis = "pub(super)")]
impl JiraClient {
    /// Load the style guidelines for Jira subtask summaries and descriptions,
    /// and prepare to create one or more subtasks under the given parent
    /// issue. Run this before calling the create_jira_subtask tool so the
    /// guidelines are in context, regardless of whether the user invoked
    /// this prompt directly or the LLM decided to load it.
    #[prompt(name = "create_jira_subtasks")]
    async fn create_subtasks_prompt(
        &self,
        Parameters(CreateSubtaskPromptInput { parent }): Parameters<CreateSubtaskPromptInput>,
    ) -> Vec<PromptMessage> {
        vec![
            PromptMessage::new_text(
                Role::User,
                format!("Create Jira subtask(s) under parent issue {parent}."),
            ),
            PromptMessage::new_text(
                Role::Assistant,
                format!(
                    "Understood. Before calling create_jira_subtask, I will follow these style \
                     guidelines for every subtask's summary and description:\n\n{SUBTASK_WRITING_GUIDELINES}"
                ),
            ),
        ]
    }
}
