---
name: jk-notebook
description: Read, search, and write notes in a jk notebook via the jk MCP server. Covers note format conventions, faux frontmatter, search strategy, and writing rules.
allowed-tools:
  - mcp__jk__rag_search
  - mcp__jk__read_note
  - mcp__jk__edit_note
  - mcp__jk__patch_note
  - mcp__jk__create_note
  - mcp__jk__list_notes
  - mcp__jk__list_tags
  - mcp__jk__reindex
  - mcp__jk__recent_journals
  - mcp__jk__append_ai_journal
---

# jk Notebook

jk is a personal knowledge base built on [zk](https://github.com/zk-org/zk) — a flat collection of interlinked Markdown notes. All interaction happens through the `jk` MCP server tools.

## Notebook layout

- **Root notes** — human-written, 4-char alphanumeric filenames (`3ycr.md`, `kbc7.md`). **Read-only.**
- **Journal entries** — human-written, `YYYY-MM-DD.md`. **Read-only.**
- **`ai/` directory** — Claude's workspace. All notes you create or edit go here.
- **`index.md`** — auto-generated link index. **Never modify.**

## Note format

Every note starts with a markdown heading followed by optional faux-frontmatter lines — plain `key: value` lines immediately after the `#` heading, before any body content. These are **not** YAML frontmatter (no `---` fences). They are parsed by zk for metadata. Section headings (`##`) can also have faux-frontmatter (most commonly `tags:`).

### Required fields

```markdown
# Title of Note
date: [Mon, Feb 23 2026](2026-02-23)
```

The date line is a markdown link where the display text is human-readable and the target is the ISO date (which also links to that day's journal entry).

### Optional fields (faux frontmatter)

These appear on lines immediately after the heading + date, before any blank line or body text:

- `tags:` — space-separated `#hashtags`. e.g. `tags: #person`, `tags: #project, #ai-generated`
- `description:` — one-line summary (used in search results)

Other `key: value` lines may be used with discretion. When editing existing notes, preserve all faux-frontmatter lines you find.

### AI notes (`ai/`)

AI notes use the same format with two additions:

```markdown
# Title of Note
date: [Mon, Feb 23 2026](../2026-02-23)
description: One-line summary.
tags: #ai-generated, #topic
```

- Date links use `../` prefix (parent directory) to reach journal entries
- Always include `description:` and `tags:` (with `#ai-generated` at minimum)
- Link *to* human notes with `[Title](../sha)` — this creates discoverable backlinks without modifying the human note
- Link to other ai notes with `[Title](sha)` (same directory)

## MCP tools

### Searching

- **`rag_search`** — primary search. Hybrid semantic + fulltext + tag boosting + recency. Supports `#tag` tokens in the query for boosting (e.g. `"puglia #travel"`), comma-separated parallel queries, and tag-only browsing.
- **`list_notes`** — browse all notes as TSV (title, path)
- **`list_tags`** — discover tag names for search queries
- **`recent_journals`** — get recent journal entries. Use `source: "ai"` to review your prior entries for continuity.

Search strategy: start with `rag_search`, then `read_note` on the top hits.

### Reading

- **`read_note`** — pass an array of relative paths (e.g. `["ai/a1b2.md", "3ycr.md"]`)

### Writing (ai/ only)

- **`create_note`** — create a new note in `ai/` with title, content, and tags. Auto-reindexes.
- **`edit_note`** — overwrite an existing `ai/` note. Takes `path` and `content`.
- **`patch_note`** — search-and-replace within an `ai/` note. Prefer this over `edit_note` for incremental changes. The `old_content` must be unique in the file.
- **`append_ai_journal`** — append to today's `ai/YYYY-MM-DD.md`. Use for session logs, observations, ongoing context.
- **`reindex`** — rebuild index and embeddings. Usually not needed (write tools auto-reindex).

## Body content conventions

- Use markdown headings, lists, and links — no tables (they render poorly in the notebook)
- Keep notes concise and scannable
- Link liberally to other notes using `[Title](path)` syntax
