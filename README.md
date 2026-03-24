`jk` is my Neovim + Rust CLI for Markdown notes. It uses a [zk](https://github.com/zk-org/zk) [LSP](https://github.com/zk-org/zk-nvim) server to manage links, supports local embeddings for RAG and live semantic search, and includes an MCP server so that agents can interface with your notebook.

<div align="center">
  <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/f52b787af9e4cfd6127fa7c88f34425b30b427bd/Screenshot%25202026-03-01%2520at%252010.10.08%25E2%2580%25AFAM.png#gh-light-mode-only" >
  <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/f52b787af9e4cfd6127fa7c88f34425b30b427bd/Screenshot%25202026-03-01%2520at%252010.09.56%25E2%2580%25AFAM.png#gh-dark-mode-only">
</div>

<details>
  <summary>More screenshots</summary>
  <div align="center">
    <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/f52b787af9e4cfd6127fa7c88f34425b30b427bd/Screenshot%25202026-03-01%2520at%252010.22.14%25E2%2580%25AFAM.png#gh-light-mode-only" >
    <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/f52b787af9e4cfd6127fa7c88f34425b30b427bd/Screenshot%25202026-03-01%2520at%252010.22.08%25E2%2580%25AFAM.png#gh-dark-mode-only">
  </div>
  <div align="center">
    <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/f52b787af9e4cfd6127fa7c88f34425b30b427bd/Screenshot%25202026-03-01%2520at%252010.31.48%25E2%2580%25AFAM.png#gh-light-mode-only" >
    <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/f52b787af9e4cfd6127fa7c88f34425b30b427bd/Screenshot%25202026-03-01%2520at%252010.31.44%25E2%2580%25AFAM.png#gh-dark-mode-only">
  </div>
  <div align="center">
    <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/f52b787af9e4cfd6127fa7c88f34425b30b427bd/Screenshot%25202026-03-01%2520at%252010.36.28%25E2%2580%25AFAM.png#gh-light-mode-only" >
    <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/f52b787af9e4cfd6127fa7c88f34425b30b427bd/Screenshot%25202026-03-01%2520at%252010.37.42%25E2%2580%25AFAM.png#gh-dark-mode-only">
  </div>
  <div align="center">
    <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/f52b787af9e4cfd6127fa7c88f34425b30b427bd/Screenshot%25202026-03-01%2520at%252010.55.19%25E2%2580%25AFAM.png#gh-light-mode-only" >
    <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/f52b787af9e4cfd6127fa7c88f34425b30b427bd/Screenshot%25202026-03-01%2520at%252010.55.09%25E2%2580%25AFAM.png#gh-dark-mode-only">
  </div>
  <div align="center">
    <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/f52b787af9e4cfd6127fa7c88f34425b30b427bd/Screenshot%25202026-03-01%2520at%252011.04.28%25E2%2580%25AFAM.png#gh-light-mode-only" >
    <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/f52b787af9e4cfd6127fa7c88f34425b30b427bd/Screenshot%25202026-03-01%2520at%252011.04.07%25E2%2580%25AFAM.png#gh-dark-mode-only">
  </div>
  <div align="center">
    <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/e8ba0d895819fdf1eb30e1f09deaf11059feb808/Screenshot%25202026-03-01%2520at%252011.39.03%25E2%2580%25AFAM.png#gh-light-mode-only" >
    <img src="https://gist.githubusercontent.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/e8ba0d895819fdf1eb30e1f09deaf11059feb808/Screenshot%25202026-03-01%2520at%252011.38.53%25E2%2580%25AFAM.png#gh-dark-mode-only">
  </div>
</details>

## Keymaps

- `<space>n` - new note (normal), new note from highlighted text (visual)
- `<space>j` - today's journal
- `<space>o` - open note
- `<space>d` - delete note
- `<space>r` - show recent notes
- `<space>f` - search notes (blended: text + semantic + tags + links)
- `<space>t` - filter notes by tag
- `<space>e` - show current note's outline
- `<space>l` - show current note's links (normal), insert link (visual)
- `<space>b` - switch buffer
- `-` - resume previous search
- `Arrow keys` - move between splits
- `Tab` - cycle through splits
- `<space>0` - new tab
- `<space>1-9` - jump to tab
- `s` - jump to text
- `K` - preview link under cursor
- `Enter` - follow link under cursor
- `Ctrl-v` - open link in vertical split
- `Ctrl-s` - open link in horizontal split
- `Ctrl-t` - open link in new tab
- `]l` / `[l` - next / prev link
- `]]` / `[[` - next / prev journal entry
- `<space>a` - toggle agent sidebar
- `<space>A` - new agent session
- `<space>x` - toggle task checkbox
- `<space>X` - TODO picker (unchecked checkboxes across all notes, sorted by recency)
- `<space>y` - yank to clipboard

## Prerequisites

- [Neovim](https://neovim.io/) >= 0.9
- [Rust](https://rustup.rs/) (for building semantic search tools)
- [zk](https://github.com/zk-org/zk): `brew install zk`
- [Ollama](https://ollama.com/) with `nomic-embed-text` for semantic search (optional)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) for the terminal split (optional)
- [Neovide](https://neovide.dev/) for GUI with smooth scrolling and hover shadows (optional)

## Install

```bash
git clone https://github.com/schoblaska/jk.git ~/.config/jk
ln -sf ~/.config/jk/bin/jk ~/.local/bin/jk
```

## Demo

```bash
cd demo
jk init
jk --gui
```

## Tag search

`<space>t` opens a tag browser. Select a tag to see every section where it appears — if the tag is in frontmatter, you see the note title; if it's inline under a heading, you see `Note > Section`. Start typing to do a full-text search scoped to tagged files, with prefix matching as you type.

Tags work at section granularity. A single note can have different tags on different headings, and each shows up independently in the picker.

<video src="https://gist.github.com/schoblaska/c252e3e7dee25e64b2be2cf589dc59fc/raw/537141ac2e4a713078a014f526ef2cfd7c7b0236/Screen%2520Recording%25202026-03-24%2520at%25206.17.59%25E2%2580%25AFPM.mov" controls></video>

## Semantic search

Requires Ollama running locally with the `nomic-embed-text` model:

```bash
ollama pull nomic-embed-text
```

Notes are re-indexed automatically on save. The first `<leader>s` search may take a moment if the index hasn't been built yet.

Embeddings are stored in `.zk/search.db`.

## MCP server

The MCP server lets AI agents search, read, and create notes in your notebook. It exposes these tools:

| Tool | Description |
|------|-------------|
| `rag_search` | Hybrid search (semantic + fulltext + tag boosting + recency) |
| `read_note` | Read one or more notes by path |
| `edit_note` | Overwrite an existing note in `ai/` |
| `create_note` | Create a new note in `ai/` |
| `list_notes` | List all notes (title + path) |
| `list_tags` | List all tags |
| `reindex` | Rebuild zk index, index.md, and embeddings |
| `recent_journals` | Get recent journal entries with full content |
| `append_ai_journal` | Append to today's AI journal (`ai/YYYY-MM-DD.md`) |

### Setup

Build the Rust tools (if you haven't already):

```bash
cd ~/.config/jk/rs && cargo build --release
```

#### Claude Code

```bash
claude mcp add jk -- ~/.config/jk/rs/target/release/jk-tools mcp /path/to/your/notebook
```

You can add multiple notebooks under different names:

```bash
claude mcp add jk-work -- ~/.config/jk/rs/target/release/jk-tools mcp ~/notes/work
claude mcp add jk-personal -- ~/.config/jk/rs/target/release/jk-tools mcp ~/notes/personal
```

#### Other MCP clients

Any MCP client that supports stdio transport can use the server. Run `jk-tools mcp <notebook-dir>` as the server command.
