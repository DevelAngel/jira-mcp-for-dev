use derive_more::Display;
use rmcp::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::key::JiraIssueKey;

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[display("Jira issue: {key}\n{fields}")]
pub struct JiraIssueOutput {
    pub(super) key: JiraIssueKey,
    pub(super) fields: JiraIssueFields,
}

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[display(
    "{summary}\n{issuetype}\n{components}\n{story_points}{description}\n",
    story_points = if let Some(sp) = &self.story_points { format!("{sp}\n") } else { "".to_owned() }
)]
pub struct JiraIssueFields {
    /// Summary of Jira issue.
    pub(super) summary: JiraSummary,
    /// Description of Jira issue.
    pub(super) description: JiraDescription,
    /// Components affected of Jira issue.
    #[serde(default)]
    pub(super) components: JiraComponentList,
    /// Issue type of Jira issue, e.g. Story, Task, Subtask.
    pub(super) issuetype: JiraIssueType,
    /// Optional Story Points value, read from a configurable custom field.
    #[serde(default, skip_deserializing)]
    pub(super) story_points: Option<JiraStoryPoints>,
}

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(transparent)]
#[display("Summary: {}", self.0)]
pub struct JiraSummary(String);

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(transparent)]
#[display("Description:\n{}", self.0)]
pub struct JiraDescription(String);

#[derive(Debug, Default, Deserialize, Display, JsonSchema, Serialize)]
#[serde(transparent)]
#[display("Components: {}", Self::format(&self.0))]
pub struct JiraComponentList(Vec<JiraComponent>);

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[display("{name}")]
pub struct JiraComponent {
    name: String,
}

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(transparent)]
#[display("Story Points: {}", self.0)]
pub struct JiraStoryPoints(pub(super) f64);

#[derive(Debug, Deserialize, Display, JsonSchema, Serialize)]
#[display(
    "Issue Type: {name}{subtask_hint}",
    subtask_hint = if self.subtask { " (cannot have subtasks of its own)" } else { "" }
)]
pub struct JiraIssueType {
    /// Name of the issue type, e.g. "Story", "Task", "Subtask".
    pub(super) name: String,
    /// Whether this issue type is itself a subtask type.
    /// If true, no further subtasks can be created under this issue.
    #[serde(default)]
    pub(super) subtask: bool,
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
