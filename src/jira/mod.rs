mod create_subtask;
mod fetch_issue;
mod key;
mod model;

#[allow(unused_imports)]
pub use create_subtask::{JiraCreateSubtaskOutput, JiraSubtaskAcceptanceCriterion};
pub use key::{JiraIssueKey, JiraIssueProject};
#[allow(unused_imports)]
pub use model::JiraIssueOutput;

use reqwest::{Client, Url};
use rmcp::ErrorData;
use rmcp::ServerHandler;
use rmcp::{prompt_handler, tool_handler};
use secrecy::SecretString;

type RmcpToolResult<T> = std::result::Result<T, ErrorData>;

#[derive(Clone, Debug)]
pub struct JiraClient {
    http: Client,
    base_url: Url,
    api_token: Option<SecretString>,
    allowed_projects: Vec<JiraIssueProject>,
    story_points_field: String,
    subtask_issuetype: String,
    non_subtaskable_issuetypes: Vec<String>,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
    prompt_router: rmcp::handler::server::router::prompt::PromptRouter<Self>,
}

#[derive(Debug)]
pub struct JiraClientBuilder {
    base_url: Url,
    api_token: Option<SecretString>,
    allowed_projects: Vec<JiraIssueProject>,
    story_points_field: Option<String>,
    subtask_issuetype: Option<String>,
    non_subtaskable_issuetypes: Vec<String>,
}

impl JiraClient {
    pub fn builder() -> JiraClientBuilder {
        JiraClientBuilder::default()
    }
}

#[tool_handler(router = self.tool_router)]
#[prompt_handler(router = self.prompt_router)]
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
            non_subtaskable_issuetypes: Vec::new(),
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

    pub fn with_non_subtaskable_issuetypes(
        mut self,
        non_subtaskable_issuetypes: impl Into<Vec<String>>,
    ) -> Self {
        self.non_subtaskable_issuetypes = non_subtaskable_issuetypes.into();
        self
    }

    pub fn build(self) -> JiraClient {
        if self.api_token.is_none() {
            tracing::warn!("no API token configured");
        }
        if self.non_subtaskable_issuetypes.is_empty() {
            tracing::warn!(
                "no non-subtaskable issuetypes configured; \
                subtasks can be created under any parent issuetype, \
                including Epics and Initiatives"
            );
        }

        let http = Client::new();
        JiraClient {
            http,
            base_url: self.base_url,
            api_token: self.api_token,
            allowed_projects: self.allowed_projects,
            story_points_field: self
                .story_points_field
                .expect("no story points field configured"),
            subtask_issuetype: self
                .subtask_issuetype
                .expect("no subtask issuetype configured"),
            non_subtaskable_issuetypes: self.non_subtaskable_issuetypes,
            tool_router: JiraClient::fetch_issue_tool_router()
                + JiraClient::create_subtask_tool_router(),
            prompt_router: JiraClient::create_subtask_prompt_router(),
        }
    }
}
