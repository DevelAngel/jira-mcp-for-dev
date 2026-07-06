mod cli;
mod jira;

use crate::cli::Cli;
use crate::jira::JiraClient;

use anyhow::Result;
use clap::Parser;

// streamable HTTP
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
        
    tracing::info!("Start streamable http server: {}", args.addr);
    serve_streamhttp(client, args.addr).await?;
    Ok(())
}

async fn serve_streamhttp(client: JiraClient, addr: SocketAddr) -> Result<()> {
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
