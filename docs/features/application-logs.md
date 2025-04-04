---
layout: default
title: Application Logs
parent: Features
nav_order: 11
---

# Application Logs

Forge generates detailed JSON-formatted logs that help with troubleshooting and understanding the application's behavior. These logs provide valuable insights into system operations and API interactions.

## Log Location and Access

Logs are stored in your application support directory with date-based filenames. The typical path looks like:

```bash
/Users/username/Library/Application Support/forge/logs/forge.log.YYYY-MM-DD
```

You can easily locate log files using the built-in command `/info`, which displays system information including the exact path to your log files.

## Viewing and Filtering Logs

To view logs in real-time with automatic updates, use the `tail` command:

```bash
tail -f /Users/username/Library/Application Support/forge/logs/forge.log.2025-03-07
```

## Formatted Log Viewing with jq

Since Forge logs are in JSON format, you can pipe them through `jq` for better readability:

```bash
tail -f /Users/username/Library/Application Support/forge/logs/forge.log.2025-03-07 | jq
```

This displays the logs in a nicely color-coded structure that's much easier to analyze, helping you quickly identify patterns, errors, or specific behavior during development and debugging.

## Log Contents

The logs contain detailed information about:

- API requests and responses
- Command executions
- File operations
- System events
- Error messages and warnings
- Performance metrics

This comprehensive logging system makes it easier to understand Forge's behavior, diagnose issues, and optimize your workflows.