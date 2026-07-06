# jira-mcp-for-dev

An MCP server to read Jira issues, for development use cases.

## Build

Build it with the following command:

```console
$ cargo build --locked
```

## MCP Streamable HTTP Server

### Terminaal 1: Start HTTP Server

```console
$ jira-mcp-for-dev mcp-http --base-url https://jira.atlassian.com --allowed-project CLOUD -vv
2026-07-06T01:27:24.559305Z  WARN jira_mcp_for_dev::jira: no API token configured
2026-07-06T01:27:24.607318Z  INFO jira_mcp_for_dev: Start streamable http server: 127.0.0.1:8000
2026-07-06T01:32:09.467448Z  INFO rmcp::transport::streamable_http_server::session::local: create new session session_id="dc2b2842-fcfa-49e8-bf5a-24273b2bf7e1"
2026-07-06T01:32:09.467752Z  INFO serve_inner: rmcp::service: Service initialized as server peer_info=Some(InitializeRequestParams { meta: None, protocol_version: ProtocolVersion("2025-03-26"), capabilities: ClientCapabilities { experimental: None, extensions: Some({}), roots: Some(RootsCapabilities { list_changed: None }), sampling: Some(SamplingCapability { tools: None, context: None }), elicitation: Some(ElicitationCapability { form: None, url: None }), tasks: None }, client_info: Implementation { name: "goose-cli", title: None, version: "1.41.0", description: None, icons: None, website_url: None } })
2026-07-06T01:32:09.468679Z  INFO serve_inner: rmcp::service: received notification notification=InitializedNotification(NotificationNoParam { method: InitializedNotificationMethod, extensions: Extensions })
2026-07-06T01:32:09.468716Z  INFO serve_inner: rmcp::handler::server: client initialized
2026-07-06T01:33:19.533230Z  INFO serve_inner: jira_mcp_for_dev::jira: jira issue fetched: CLOUD-12377
2026-07-06T01:33:39.812158Z  WARN serve_inner: rmcp::service: response error id=4 error=ErrorData { code: ErrorCode(-32602), message: "Jira issue PROJ-123 is not allowed", data: None }
```

### Terminaal 2: Goose AI

```console
$ goose configure

┌   goose-configure
│
◇  What would you like to configure?
│  Add Extension
│
◇  What type of extension would you like to add?
│  Remote Extension (Streamable HTTP)
│
◇  What would you like to call this extension?
│  jira-mcp-for-dev
│
◇  What is the Streaming HTTP endpoint URI?
│  http://localhost:8000/mcp
│
◇  Please set the timeout for this tool (in secs):
│  300
│
◇  Enter a description for this extension:
│  Fetch Jira issues
│
◇  Would you like to add custom headers?
│  No
│
└  Added jira-mcp-for-dev extension
```

```console
$ goose

    __( O)>  ● new session · anthropic claude-sonnet-5
   \____)    20260706_7 · /home/develangel/workspace3/rust/jira-mcp-for-dev
     L L     goose is ready
  ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ 0% 0/1.0M
> What MCP servers are configured?

Based on my current active extensions, I only have access to one MCP server:

## Configured MCP Server

- **jira-mcp-for-dev** — provides a tool to fetch Jira issue summaries and descriptions (`fetch_jira_issue`)

This is the only extension currently active in my session. I don't have direct visibility into your broader MCP configuration file (e.g., `~/.config/goose/config.yaml` or similar) beyond what's been loaded as active extensions for this conversation.

If you'd like, I can:
1. Look at your working directory (`/home/develangel/workspace3/rust/jira-mcp-for-dev`) — though I'd need shell/file access tools enabled to do that, which I currently don't have.
2. You could share your Goose config file contents directly and I can help interpret them.

Would you like me to enable additional extensions (like a developer/shell tool) to inspect the actual config file?
  ⏱ 6.29s
  ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ 0% 1k/1.0M
> What do you know about the Jira Ticket CLOUD-12377 -- in one sentence?


  ────────────────────────────────────────
  ▸ fetch_jira_issue jira-mcp-for-dev
    key: CLOUD-12377

CLOUD-12377 is a feature request to extend the "User Count" monitoring capability in the Admin Hub—currently limited to Enterprise customers—to also be available for organizations on the Standard and Premium plans.
  ⏱ 4.99s
  ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ 0% 2k/1.0M
> What about Jira PROJ-123?


  ────────────────────────────────────────
  ▸ fetch_jira_issue jira-mcp-for-dev
    key: PROJ-123

    -32602: Jira issue PROJ-123 is not allowed
It looks like I'm not able to access **PROJ-123** — the server returned an error indicating that this issue is **not allowed** (likely restricted by project-level permissions or an allow-list configured for this Jira MCP server, which may only permit access to specific projects like `CLOUD`).

If you have access to a ticket in an allowed project (e.g., `CLOUD-XXXXX`), I'd be happy to look that up instead.
  ⏱ 5.33s
  ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ 0% 2k/1.0M
```

## MCP IO Server

### Example Usage in Goose AI

```console
$ goose configure

┌   goose-configure
│
◇  What would you like to configure?
│  Add Extension
│
◇  What type of extension would you like to add?
│  Command-line Extension
│
◇  What would you like to call this extension?
│  jira-mcp-for-dev
│
◇  What command should be run?
│  jira-mcp-for-dev mcp-io --base-url https://jira.atlassian.com --allowed-project CLOUD
│
◇  Please set the timeout for this tool (in secs):
│  300
│
◇  Enter a description for this extension:
│  Fetch Jira issues
│
◇  Would you like to add environment variables?
│  No
│
└  Added jira-mcp-for-dev extension
```

```console
$ goose

    __( O)>  ● new session · anthropic claude-sonnet-5
   \____)    20260706_6 · ~/jira-mcp-for-dev
     L L     goose is ready
  ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ 0% 0/1.0M
> What MCP servers are configured?

I don't have a way to inspect your full MCP client configuration directly — my visibility is limited to the extensions that are currently active in this session. Right now, I only have access to:

- **jira-mcp-for-dev** — provides a tool to fetch summary and description of Jira issues (`fetch_jira_issue`)

If you want to know what other MCP servers are configured on your system (e.g., in a config file like `~/.config/goose/config.yaml` or similar), I'd need to inspect that file directly. Since I'm currently in the directory `/home/develangel/workspace3/rust/jira-mcp-for-dev`, would you like me to look for a configuration file to check for other configured servers? I don't currently have a file-system/shell extension enabled, so let me know if you'd like me to try enabling one to check.
  ⏱ 4.41s
  ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ 0% 1k/1.0M
> What do you know about the Jira Ticket CLOUD-12377 -- in one sentence?


  ────────────────────────────────────────
  ▸ fetch_jira_issue jira-mcp-for-dev
    key: CLOUD-12377

CLOUD-12377 is a feature request to extend the "User Count" monitoring feature—currently exclusive to Enterprise customers—so that Standard and Premium plan admins can also monitor their organization's user counts from the Admin Hub.
  ⏱ 3.91s
  ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ 0% 2k/1.0M
> What about Jira PROJ-123?


  ────────────────────────────────────────
  ▸ fetch_jira_issue jira-mcp-for-dev
    key: PROJ-123

    -32602: Jira issue PROJ-123 is not allowed
It looks like access to **PROJ-123** isn't permitted — the Jira MCP server returned an error saying that issue "is not allowed" (likely restricted to specific projects, such as `CLOUD`, based on this server's configuration). If you have a valid issue key from an allowed project, I'd be happy to look it up.
  ⏱ 4.04s
  ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ 0% 2k/1.0M
```

## Fetch Jira Issue Directly

Bypass the MCP server.

### Example Usage

```console
$ jira-mcp-for-dev fetch-issue --base-url https://jira.atlassian.com --allowed-project CLOUD CLOUD-12377
jira issue: CLOUD-12377
summary: Allow non-Enterprise organization admins to use ...
description:
We’d like to request that the *User Count* feature be made available for ...
```

(Summary and description was shorten.)

### Not in Allow List (wrong project #1)

```console
$ jira-mcp-for-dev fetch-issue --base-url https://jira.atlassian.com --allowed-project PROJ CLOUD-12377
Error: failed to fetch Jira issue CLOUD-12377

Caused by:
    -32602: Jira issue CLOUD-12377 is not allowed
```

### Not in Allow List (wrong project #2)

```console
$ jira-mcp-for-dev fetch-issue --base-url https://jira.atlassian.com --allowed-project CLO CLOUD-12377
Error: failed to fetch Jira issue CLOUD-12377

Caused by:
    -32602: Jira issue CLOUD-12377 is not allowed
```

### Not in Allow List (empty list)

```console
$ jira-mcp-for-dev fetch-issue --base-url https://jira.atlassian.com CLOUD-12377
Error: failed to fetch Jira issue CLOUD-12377

Caused by:
    -32602: Jira issue CLOUD-12377 is not allowed
```
