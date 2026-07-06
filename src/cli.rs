use crate::jira::JiraIssueKeyPrefix;

use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use reqwest::Url;
use secrecy::SecretString;

use std::net::SocketAddr;

#[derive(Debug, Parser)]
#[command(author, version, about)]
#[command(after_help = "If no subcommand is given, io is used by default.")]
pub(crate) struct Cli {
    // transport mode
    #[command(subcommand)]
    pub command: Option<Transport>,

    // jira options
    #[command(flatten)]
    pub jira: JiraArgs,

    // verbose and quiet flag handling
    #[command(flatten)]
    pub verbosity: Verbosity<WarnLevel>,
}

#[derive(Debug, Subcommand)]
pub enum Transport {
    /// Runs the MCP server using the stdio transport
    Io,
    /// Runs the MCP server using the Streamable HTTP transport
    Http {
        /// MCP server address
        #[arg(
            long,
            env = "JIRA_MCP_ADDRESS",
            value_name = "ADDRESS:PORT",
            default_value = "127.0.0.1:8000"
        )]
        addr: SocketAddr,
    },
}

#[derive(Args, Clone, Debug)]
pub struct JiraArgs {
    /// Allowed Jira issue key prefixes, e.g. PROJ.
    /// Can be repeated or comma-separated.
    #[arg(
        global = true,
        long = "allowed-prefix",
        env = "JIRA_ALLOWED_KEY_PREFIXES",
        value_name = "KEY_PREFIX",
        value_delimiter = ','
    )]
    pub allowed_key_prefixes: Vec<JiraIssueKeyPrefix>,

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

impl Default for Transport {
    fn default() -> Self {
        Self::Io
    }
}
