use anyhow::{Error, Result, anyhow};
use derive_more::{Deref, Display};
use regex::Regex;
use reqwest::{Client, Url};
use rmcp::ErrorData;
use rmcp::Json;
use rmcp::ServerHandler;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{PromptMessage, Role};
use rmcp::schemars;
use rmcp::{prompt, prompt_handler, prompt_router, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;

type RmcpToolResult<T> = std::result::Result<T, ErrorData>;

/// Style guidelines for writing Jira subtask summaries and descriptions.
/// Kept out of `create_jira_subtask`'s tool description so it is only
/// loaded into the LLM's context when actually needed, not on every
/// tool listing.
const SUBTASK_WRITING_GUIDELINES: &str =
    include_str!("../resources/create-jira-subtasks.md");

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
    /// Whether to also fetch the optional Story Points value.
    /// Defaults to false, since it lives in an instance-specific custom
    /// field and is not always needed.
    #[serde(default)]
    include_story_points: bool,
}

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[display("Jira issue: {key}\n{fields}")]
pub struct JiraIssueOutput {
    key: JiraIssueKey,
    fields: JiraIssueFields,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
struct CreateSubtaskPromptInput {
    /// Key of the parent issue the subtask(s) will be created under, e.g. PROJ-123.
    parent: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
struct JiraCreateSubtaskInput {
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

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[display(
    "{summary}\n{issuetype}\n{components}\n{story_points}{description}\n",
    story_points = if let Some(sp) = &self.story_points { format!("{sp}\n") } else { "".to_owned() }
)]
struct JiraIssueFields {
    /// Summary of Jira issue.
    summary: JiraSummary,
    /// Description of Jira issue.
    description: JiraDescription,
    /// Components affected of Jira issue.
    #[serde(default)]
    components: JiraComponentList,
    /// Issue type of Jira issue, e.g. Story, Task, Subtask.
    issuetype: JiraIssueType,
    /// Optional Story Points value, read from a configurable custom field.
    #[serde(default, skip_deserializing)]
    story_points: Option<JiraStoryPoints>,
}

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(transparent)]
#[display("Summary: {}", self.0)]
struct JiraSummary(String);

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(transparent)]
#[display("Description:\n{}", self.0)]
struct JiraDescription(String);

#[derive(Debug, Default, Deserialize, Display, JsonSchema, Serialize)]
#[serde(transparent)]
#[display("Components: {}", Self::format(&self.0))]
struct JiraComponentList(Vec<JiraComponent>);

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[display("{name}")]
struct JiraComponent {
    name: String,
}

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(transparent)]
#[display("Story Points: {}", self.0)]
struct JiraStoryPoints(f64);

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[display(
    "Issue Type: {name}{subtask_hint}",
    subtask_hint = if self.subtask { " (cannot have subtasks of its own)" } else { "" }
)]
struct JiraIssueType {
    /// Name of the issue type, e.g. "Story", "Task", "Subtask".
    name: String,
    /// Whether this issue type is itself a subtask type.
    /// If true, no further subtasks can be created under this issue.
    #[serde(default)]
    subtask: bool,
}

#[derive(Clone, Debug)]
pub struct JiraClient {
    http: Client,
    base_url: Url,
    api_token: Option<SecretString>,
    allowed_projects: Vec<JiraIssueProject>,
    story_points_field: String,
    subtask_issuetype: String,
}

#[derive(Debug)]
pub struct JiraClientBuilder {
    base_url: Url,
    api_token: Option<SecretString>,
    allowed_projects: Vec<JiraIssueProject>,
    story_points_field: Option<String>,
    subtask_issuetype: Option<String>,
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

/// Render the structured subtask description fields into Jira wiki markup.
/// Keeping this deterministic on the server removes an entire class of
/// LLM markup mistakes (wrong heading, malformed code blocks, stray
/// "Feature:"/"Scenario:" keywords).
fn render_description(
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

impl JiraClient {
    pub fn builder() -> JiraClientBuilder {
        JiraClientBuilder::default()
    }
}

#[tool_router]
impl JiraClient {
    #[tool(
        name = "fetch_jira_issue",
        description = "Fetch summary, description, and components of a jira issue. Set include_story_points=true to also fetch the Story Points value",
        annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true),
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
        url.query_pairs_mut()
            .append_pair("fields", &fields_param);

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

    #[tool(
        name = "create_jira_subtask",
        description = "Create a subtask under a parent Jira issue. Use the create_jira_subtasks prompt first to load the summary/narrative/acceptance-criteria style guidelines.",
        annotations(read_only_hint = false, destructive_hint = false, idempotent_hint = false),
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
            .create_subtask_in_jira(&parent, &summary, &narrative, &acceptance_criteria, &out_of_scope)
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

#[prompt_router]
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

#[tool_handler]
#[prompt_handler]
impl ServerHandler for JiraClient {}

impl Default for JiraClientBuilder {
    fn default() -> Self {
        let base_url = "http://localhost:8080".parse().unwrap();
        Self {
            base_url,
            api_token: None,
            allowed_projects: Vec::new(),
            story_points_field: None,
            subtask_issuetype: None,
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

    pub fn with_story_points_field(mut self, story_points_field: impl Into<String>) -> Self {
        self.story_points_field = Some(story_points_field.into());
        self
    }

    pub fn with_subtask_issuetype(mut self, subtask_issuetype: impl Into<String>) -> Self {
        self.subtask_issuetype = Some(subtask_issuetype.into());
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
            story_points_field: self.story_points_field.expect("no story points field configured"),
            subtask_issuetype: self.subtask_issuetype.expect("no subtask issuetype configured"),
        }
    }
}

impl JiraComponentList {
    fn format(components: &[JiraComponent]) -> String {
        if components.is_empty() {
            "none".to_string()
        } else {
            components
                .iter()
                .map(|c| c.name.as_str().trim_end())
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}
