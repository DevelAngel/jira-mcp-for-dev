mod cli;
mod jira;

use crate::cli::Cli;
use crate::jira::JiraClient;

use anyhow::{anyhow, Context, Result};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .init();

    tracing::info!("fetch jira issue: {}", args.key);
    if !args.key.is_allowed(&args.allowed_key_prefixes) {
        return Err(anyhow!("{} not allowed", args.key))
    }

    let client = if let Some(api_token) = args.api_token {
        JiraClient::builder()
            .with_base_url(args.base_url)
            .with_api_token(api_token)
            .build()
    } else {
        JiraClient::builder()
            .with_base_url(args.base_url)
            .build()
    };
    let ticket = client
        .get_issue(&args.key)
        .await
        .with_context(|| format!("failed to fetch Jira issue {}", args.key))?;

    println!("jira issue: {}", ticket.key);
    println!("summary: {}", ticket.fields.summary);
    println!("description:");
    println!("{}", ticket.fields.description);
    Ok(())
}

