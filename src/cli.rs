use crate::jira::JiraIssueKeyPrefix;

use clap::Parser;
use clap_verbosity_flag::Verbosity;
use reqwest::Url;
use secrecy::SecretString;

use std::net::SocketAddr;

#[derive(Debug, Parser)]
#[command(version, about = "Fetch summary and description for a Jira issue")]
pub(crate) struct Cli {
    // verbose and quiet flag handling
    #[command(flatten)]
    pub verbosity: Verbosity,
    /// Allowed Jira issue key prefixes, e.g. PROJ.
    /// Can be repeated or comma-separated.
    #[arg(
        long = "allowed-prefix",
        env = "JIRA_ALLOWED_KEY_PREFIXES",
        value_name = "KEY_PREFIX",
        value_delimiter = ','
    )]
    pub allowed_key_prefixes: Vec<JiraIssueKeyPrefix>,
    /// Jira Base URL, e.g. https://jira.example.com
    #[arg(long, env = "JIRA_BASE_URL")]
    pub base_url: Url,
    /// Jira API token
    #[arg(long, env = "JIRA_API_TOKEN")]
    pub api_token: Option<SecretString>,
    /// MCP server address
    #[arg(long, env = "JIRA_MCP_ADDRESS", default_value = "127.0.0.1:8000")]
    pub addr: SocketAddr,
}
