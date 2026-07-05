# jira-mcp-for-dev

An MCP server to read Jira issues, for development use cases.

## Version 0.1.0 -- CLI

Build it with the following command:

```console
$ cargo build --locked
```

Example usage:

```console
$ ./target/debug/jira-mcp-for-dev --base-url https://jira.atlassian.com CLOUD-12377
jira issue: CLOUD-12377
summary: Allow non-Enterprise organization admins to use ...
description:
We’d like to request that the *User Count* feature be made available for ...
```

(Summary and description was shorten.)
