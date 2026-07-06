# jira-mcp-for-dev

An MCP server to read Jira issues, for development use cases.

## Version 0.1.1 -- CLI

Build it with the following command:

```console
$ cargo build --locked
```

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
