use crate::jira::{JiraIssueKey, JiraIssueProject, JiraSubtaskAcceptanceCriterion};

use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use reqwest::Url;
use secrecy::SecretString;
use serde::de::DeserializeOwned;

use std::net::SocketAddr;

/// Parses a CLI argument as JSON, so malformed input is rejected at parse
/// time rather than later when building the request.
fn parse_json<T: DeserializeOwned>(raw: &str) -> Result<T, String> {
    serde_json::from_str(raw).map_err(|err| format!("invalid JSON: {err}"))
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
#[command(after_help = "If no subcommand is given, mcp-io is used by default.")]
pub(crate) struct Cli {
    // transport mode
    #[command(subcommand)]
    pub command: Option<Command>,

    // jira options
    #[command(flatten)]
    pub jira: JiraArgs,

    // verbose and quiet flag handling
    #[command(flatten)]
    pub verbosity: Verbosity<WarnLevel>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Runs the MCP server using the stdio transport
    McpIo,
    /// Runs the MCP server using the Streamable HTTP transport
    McpHttp {
        /// MCP server address
        #[arg(
            long,
            env = "JIRA_MCP_ADDRESS",
            value_name = "ADDRESS:PORT",
            default_value = "127.0.0.1:8000"
        )]
        addr: SocketAddr,
        /// Allowed Origins.
        /// Can be repeated or comma-separated.
        #[arg(
            long = "allowed-origin",
            env = "JIRA_ALLOWED_ORIGINS",
            value_name = "BASE_URL",
            value_delimiter = ','
        )]
        allowed_origins: Vec<Url>,
    },
    /// Downloads a Jira ticket directly, bypassing the MCP server
    FetchIssue {
        /// Jira issue key, e.g. PROJ-123
        key: JiraIssueKey,
        /// Also fetch the optional Story Points value
        #[arg(long)]
        include_story_points: bool,
    },
    /// Creates a subtask under a parent Jira issue, bypassing the MCP server
    CreateSubtask {
        /// Key of the parent Jira issue, e.g. PROJ-123
        parent: JiraIssueKey,
        /// Summary of the new subtask
        summary: String,
        /// Narrative context of the new subtask (one to two short paragraphs)
        narrative: String,
        /// Acceptance criteria as a JSON array of
        /// {"scenario": ..., "steps": ...} objects
        #[arg(long, value_name = "JSON", value_parser = parse_json::<Vec<JiraSubtaskAcceptanceCriterion>>)]
        acceptance_criteria: Vec<JiraSubtaskAcceptanceCriterion>,
        /// An out-of-scope item; repeat the flag for multiple items,
        /// e.g. --out-of-scope foo --out-of-scope bar
        #[arg(long)]
        out_of_scope: Vec<String>,
    },
    /// Replaces the description of an existing subtask, bypassing the MCP
    /// server. Refuses to run on any issue type other than a subtask.
    UpdateSubtaskDescription {
        /// Key of the subtask to update, e.g. PROJ-123
        key: JiraIssueKey,
        /// Narrative context of the updated subtask (one to two short
        /// paragraphs)
        narrative: String,
        /// Acceptance criteria as a JSON array of
        /// {"scenario": ..., "steps": ...} objects
        #[arg(long, value_name = "JSON", value_parser = parse_json::<Vec<JiraSubtaskAcceptanceCriterion>>)]
        acceptance_criteria: Vec<JiraSubtaskAcceptanceCriterion>,
        /// An out-of-scope item; repeat the flag for multiple items,
        /// e.g. --out-of-scope foo --out-of-scope bar
        #[arg(long)]
        out_of_scope: Vec<String>,
    },
    /// Replaces the description of an existing non-subtask issue, bypassing
    /// the MCP server. Refuses to run on subtasks.
    UpdateIssueDescription {
        /// Key of the issue to update, e.g. PROJ-123
        key: JiraIssueKey,
        /// Narrative context of the updated issue (one to two short
        /// paragraphs)
        narrative: String,
        /// Acceptance criteria as a JSON array of
        /// {"scenario": ..., "steps": ...} objects
        #[arg(long, value_name = "JSON", value_parser = parse_json::<Vec<JiraSubtaskAcceptanceCriterion>>)]
        acceptance_criteria: Vec<JiraSubtaskAcceptanceCriterion>,
        /// An out-of-scope item; repeat the flag for multiple items,
        /// e.g. --out-of-scope foo --out-of-scope bar
        #[arg(long)]
        out_of_scope: Vec<String>,
    },
}

#[derive(Args, Clone, Debug)]
pub struct JiraArgs {
    /// Allowed Jira issue projects, e.g. PROJ.
    /// Can be repeated or comma-separated.
    #[arg(
        global = true,
        long = "allowed-project",
        env = "JIRA_ALLOWED_PROJECTS",
        value_name = "KEY_PREFIX",
        value_delimiter = ','
    )]
    pub allowed_projects: Vec<JiraIssueProject>,

    /// Jira Base URL, e.g. "https://jira.atlassian.com".
    ///
    /// Defaults to localhost as a safety measure, so sensitive information
    /// is not accidentally sent to an external Jira instance.
    #[arg(
        global = true,
        long,
        env = "JIRA_BASE_URL",
        value_name = "URL",
        default_value = "http://localhost:8000"
    )]
    pub base_url: Url,

    /// Jira API token
    #[arg(global = true, long, env = "JIRA_API_TOKEN", value_name = "TOKEN")]
    pub api_token: Option<SecretString>,

    /// Custom field ID used for the optional Story Points value.
    ///
    /// Story Points is not a standard Jira field; its ID varies per instance.
    #[arg(
        global = true,
        long = "story-points-field",
        env = "JIRA_STORY_POINTS_FIELD",
        value_name = "FIELD_ID",
        default_value = "customfield_10106"
    )]
    pub story_points_field: String,

    /// Issue type name used when creating subtasks.
    ///
    /// Varies per Jira instance/locale, e.g. "Subtask" vs. "Sub-task".
    #[arg(
        global = true,
        long = "subtask-issuetype",
        env = "JIRA_SUBTASK_ISSUETYPE",
        value_name = "ISSUETYPE_NAME",
        default_value = "Sub-task"
    )]
    pub subtask_issuetype: String,

    /// Parent issue type names that must never receive subtasks, e.g.
    /// "Epic", "Initiative".
    ///
    /// Varies per Jira instance/locale. Matched case-insensitively.
    #[arg(
        global = true,
        long = "non-subtaskable-issuetype",
        env = "JIRA_NON_SUBTASKABLE_ISSUETYPES",
        value_name = "ISSUETYPE_NAME",
        value_delimiter = ',',
        default_value = "Epic,Initiative"
    )]
    pub non_subtaskable_issuetypes: Vec<String>,
}

impl Default for Command {
    fn default() -> Self {
        Self::McpIo
    }
}
