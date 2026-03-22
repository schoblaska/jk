# jk Notebook Tools

All tools are available via the `jk` MCP server.

## rag_search

Hybrid search combining semantic similarity, fulltext (BM25), tag boosting, and recency signals.

- **query**: Natural language with optional `#tag` tokens for boosting
  - `"puglia #travel"` — finds notes about Puglia, boosts notes tagged #travel
  - `"machine learning, neural networks"` — comma-separated queries run in parallel and merge
  - `"#travel #italy"` — tag-only query to browse by topic
- **limit** (optional): Max results, default 15
- **expand_links** (optional): Include one-hop linked notes, default true

Results include scores, file paths, dates, tags, and excerpts. Use `read_note` to get full content for the most relevant hits.

## read_note

Read the full contents of one or more notes. Pass an array of relative paths (e.g. `["ai/a1b2.md", "travel/c3d4.md"]`). Results are separated by `---` headers.

## edit_note

Overwrite an existing note in `ai/`. Takes `path` and `content`. Human notes (root-level) are read-only. Auto-reindexes after editing.

## create_note

Create a new note in `ai/` with title, optional content, and optional tags. Auto-reindexes.

## list_notes

List all notes as TSV (title, path). Useful for browsing.

## list_tags

List all tags used in the notebook. Use to discover tag names for `rag_search` queries.

## reindex

Rebuild the zk index, regenerate `index.md`, and update semantic embeddings. Pass specific file paths for incremental reindex, or omit for full reindex.

## Search Strategy

1. **index.md** — read first for a full map of all notes by title
2. **rag_search** — your primary search tool for everything: keyword lookup, conceptual queries, tag-filtered browsing, finding related notes
3. **read_note** — dig into the top results from rag_search
