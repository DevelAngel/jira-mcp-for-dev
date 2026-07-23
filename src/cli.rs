use crate::jira::{JiraIssueKey, JiraIssueProject};

use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use reqwest::Url;
use secrecy::SecretString;

use std::net::SocketAddr;

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

    /// Jira Base URL
    #[arg(
        global = true,
        long,
        env = "JIRA_BASE_URL",
        value_name = "URL",
        default_value = "https://jira.atlassian.com"
    )]
    pub base_url: Url,

    /// Jira API token
    #[arg(global = true, long, env = "JIRA_API_TOKEN", value_name = "TOKEN")]
    pub api_token: Option<SecretString>,
}

impl Default for Command {
    fn default() -> Self {
        Self::McpIo
    }
}
