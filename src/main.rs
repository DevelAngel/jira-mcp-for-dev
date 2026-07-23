mod cli;
mod jira;

use crate::cli::{Cli, Command};
use crate::jira::{JiraClient, JiraIssueKey, JiraSubtaskAcceptanceCriterion};

use anyhow::{Context, Result};
use clap::Parser;

// io transport
use rmcp::ServiceExt;
use rmcp::transport;

// streamable HTTP transport
use axum::Router;
use reqwest::Url;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

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
            .with_story_points_field(cli.jira.story_points_field)
            .with_subtask_issuetype(cli.jira.subtask_issuetype)
            .with_non_subtaskable_issuetypes(cli.jira.non_subtaskable_issuetypes)
            .with_api_token(api_token)
            .build()
    } else {
        JiraClient::builder()
            .with_base_url(cli.jira.base_url)
            .with_allowed_projects(cli.jira.allowed_projects)
            .with_story_points_field(cli.jira.story_points_field)
            .with_subtask_issuetype(cli.jira.subtask_issuetype)
            .with_non_subtaskable_issuetypes(cli.jira.non_subtaskable_issuetypes)
            .build()
    };

    match cli.command.unwrap_or_default() {
        Command::McpIo => run_mcp_io_server(client).await,
        Command::McpHttp {
            addr,
            allowed_origins,
        } => run_mcp_http_server(client, addr, &allowed_origins).await,
        Command::FetchIssue {
            key,
            include_story_points,
        } => fetch_issue(client, key, include_story_points).await,
        Command::CreateSubtask {
            parent,
            summary,
            narrative,
            acceptance_criteria,
            out_of_scope,
        } => {
            create_subtask(
                client,
                parent,
                summary,
                narrative,
                acceptance_criteria,
                out_of_scope,
            )
            .await
        }
        Command::UpdateSubtaskDescription {
            key,
            narrative,
            acceptance_criteria,
            out_of_scope,
        } => {
            update_subtask_description(client, key, narrative, acceptance_criteria, out_of_scope)
                .await
        }
    }?;

    Ok(())
}

async fn run_mcp_io_server(client: JiraClient) -> Result<()> {
    tracing::info!("Start stdio server");
    let service = client.serve(transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}

async fn run_mcp_http_server(
    client: JiraClient,
    addr: SocketAddr,
    allowed_origins: &[Url],
) -> Result<()> {
    tracing::info!("Start streamable http server: {}", addr);
    if allowed_origins.is_empty() {
        tracing::warn!("No allowed origins");
    } else {
        let allowed_origins: Vec<_> = allowed_origins.iter().map(|url| url.to_string()).collect();
        tracing::info!("Allowed origins: {}", allowed_origins.join(", "));
    }
    let allowed_hosts: Vec<_> = allowed_origins
        .iter()
        .map(|url| url.host_str().expect("url have no host"))
        .collect();
    if !allowed_hosts.is_empty() {
        tracing::info!("Allowed hosts: {}", allowed_hosts.join(", "));
    }

    let ct = CancellationToken::new();

    let service = StreamableHttpService::new(
        move || Ok(client.clone()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default()
            .with_allowed_origins(allowed_origins.into_iter().map(|url| url.to_string()))
            .with_allowed_hosts(allowed_hosts)
            .with_cancellation_token(ct.child_token()),
    );

    let router = Router::new()
        .nest_service("/mcp", service)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
    let tcp_listener = TcpListener::bind(addr).await?;
    let _ = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async move {
            signal::ctrl_c().await.unwrap();
            ct.cancel();
        })
        .await;
    Ok(())
}

async fn fetch_issue(
    client: JiraClient,
    key: JiraIssueKey,
    include_story_points: bool,
) -> Result<()> {
    let issue = client
        .fetch_issue_from_jira(&key, include_story_points)
        .await
        .with_context(|| format!("failed to fetch Jira issue {}", key))?;
    println!("{issue}");
    Ok(())
}

async fn create_subtask(
    client: JiraClient,
    parent: JiraIssueKey,
    summary: String,
    narrative: String,
    acceptance_criteria: Vec<JiraSubtaskAcceptanceCriterion>,
    out_of_scope: Vec<String>,
) -> Result<()> {
    let subtask = client
        .create_subtask_in_jira(
            &parent,
            &summary,
            &narrative,
            &acceptance_criteria,
            &out_of_scope,
        )
        .await
        .with_context(|| format!("failed to create Jira subtask under {}", parent))?;
    println!("{subtask}");
    Ok(())
}

async fn update_subtask_description(
    client: JiraClient,
    key: JiraIssueKey,
    narrative: String,
    acceptance_criteria: Vec<JiraSubtaskAcceptanceCriterion>,
    out_of_scope: Vec<String>,
) -> Result<()> {
    let subtask = client
        .update_subtask_description_in_jira(&key, &narrative, &acceptance_criteria, &out_of_scope)
        .await
        .with_context(|| format!("failed to update Jira subtask description for {}", key))?;
    println!("{subtask}");
    Ok(())
}
