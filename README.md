# jira-mcp-for-dev

An MCP server to read Jira issues, for development use cases.

## Version 0.2.0 -- MCP IO Server

Build it with the following command:

```console
$ cargo build --locked
```

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
│  jira-mcp-for-dev --base-url https://jira.atlassian.com --allowed-prefix CLOUD
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

## Version 0.1.1 -- CLI

### Example Usage

```console
$ ./target/debug/jira-mcp-for-dev --base-url https://jira.atlassian.com --allowed-prefix CLOUD CLOUD-12377
jira issue: CLOUD-12377
summary: Allow non-Enterprise organization admins to use ...
description:
We’d like to request that the *User Count* feature be made available for ...
```

(Summary and description was shorten.)

### Not in Allow List (wrong prefix)

```console
$ ./target/debug/jira-mcp-for-dev --base-url https://jira.atlassian.com --allowed-prefix PROJ CLOUD-12377
Error: CLOUD-12377 not allowed
```

### Not in Allow List (empty list)

```console
./target/debug/jira-mcp-for-dev --base-url https://jira.atlassian.com CLOUD-12377
Error: CLOUD-12377 not allowed
```
