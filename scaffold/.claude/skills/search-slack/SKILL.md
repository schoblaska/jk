---
name: search-slack
description: Search Slack messages via slackdump CLI
allowed-tools:
  - Bash(jk search-slack *)
---

# Search Slack

Search Slack messages using `jk search-slack`.

## Usage

```
jk search-slack "query" [limit]
```

- **First argument:** search query (required)
- **Second argument:** max results (optional, default 50)
- **Output:** TSV (`channel<TAB>datetime<TAB>text`)

## Requirements

Requires `slackdump` CLI to be installed and configured.
