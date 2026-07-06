mod cli;
mod jira;

use crate::cli::{Cli, Command};
use crate::jira::{JiraClient, JiraIssueKey};

use anyhow::{Context, Result};
use clap::Parser;

// io transport
use rmcp::ServiceExt;
use rmcp::transport;

// streamable HTTP transport
use axum::Router;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(cli.verbosity)
        .init();

    let client = if let Some(api_token) = cli.jira.api_token {
        JiraClient::builder()
            .with_base_url(cli.jira.base_url)
            .with_allowed_projects(cli.jira.allowed_projects)
            .with_api_token(api_token)
            .build()
    } else {
        JiraClient::builder()
            .with_base_url(cli.jira.base_url)
            .with_allowed_projects(cli.jira.allowed_projects)
            .build()
    };

    match cli.command.unwrap_or_default() {
        Command::McpIo => run_mcp_io_server(client).await,
        Command::McpHttp { addr } => run_mcp_http_server(client, addr).await,
        Command::FetchIssue { key } => fetch_issue(client, key).await,
    }?;

    Ok(())
}

async fn run_mcp_io_server(client: JiraClient) -> Result<()> {
    tracing::info!("Start stdio server");
    let service = client.serve(transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}

async fn run_mcp_http_server(client: JiraClient, addr: SocketAddr) -> Result<()> {
    tracing::info!("Start streamable http server: {}", addr);
    let ct = CancellationToken::new();

    let service = StreamableHttpService::new(
        move || Ok(client.clone()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_cancellation_token(ct.child_token()),
    );

    let router = Router::new().nest_service("/mcp", service);
    let tcp_listener = TcpListener::bind(addr).await?;
    let _ = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async move {
            signal::ctrl_c().await.unwrap();
            ct.cancel();
        })
        .await;
    Ok(())
}

async fn fetch_issue(client: JiraClient, key: JiraIssueKey) -> Result<()> {
    let issue = client
        .fetch_issue_from_jira(&key)
        .await
        .with_context(|| format!("failed to fetch Jira issue {}", key))?;
    println!("{issue}");
    Ok(())
}
