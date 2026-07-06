mod cli;
mod jira;

use crate::cli::Cli;
use crate::jira::JiraClient;

use anyhow::Result;
use clap::Parser;
use rmcp::transport;
use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .init();

    let client = if let Some(api_token) = args.api_token {
        JiraClient::builder()
            .with_base_url(args.base_url)
            .with_allowed_key_prefixes(args.allowed_key_prefixes)
            .with_api_token(api_token)
            .build()
    } else {
        JiraClient::builder()
            .with_base_url(args.base_url)
            .with_allowed_key_prefixes(args.allowed_key_prefixes)
            .build()
    };
        
    serve_io(client).await?;
    Ok(())
}

async fn serve_io(client: JiraClient) -> Result<()> {
    let service = client.serve(transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
