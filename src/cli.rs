use crate::jira::JiraIssueKey;

use clap::Parser;
use clap_verbosity_flag::Verbosity;
use reqwest::Url;
use secrecy::SecretString;

#[derive(Debug, Parser)]
#[command(about = "Fetch summary and description for a Jira issue")]
pub(crate) struct Cli {
    // verbose and quiet flag handling
    #[command(flatten)]
    pub verbosity: Verbosity,
    /// Jira issue key, e.g. PROJ-123
    pub key: JiraIssueKey,
    /// Jira Base URL, e.g. https://jira.example.com
    #[arg(long, env = "JIRA_BASE_URL")]
    pub base_url: Url,
    /// Jira API token
    #[arg(long, env = "JIRA_API_TOKEN")]
    pub api_token: Option<SecretString>,
}
