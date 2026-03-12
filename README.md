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
- `<space>f` - search notes (grep)
- `<space>s` - search notes (semantic)
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
- `<space>y` - yank to clipboard

## Prerequisites

- [Neovim](https://neovim.io/) >= 0.9
- [Rust](https://rustup.rs/) (for building semantic search tools)
- [zk](https://github.com/zk-org/zk): `brew install zk`
- [Ollama](https://ollama.com/) with `nomic-embed-text` for semantic search (optional)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) for the terminal split (optional)
- [Neovide](https://neovide.dev/) for GUI with smooth scrolling and hover shadows

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

## Semantic search

Requires Ollama running locally with the `nomic-embed-text` model:

```bash
ollama pull nomic-embed-text
```

Notes are re-indexed automatically on save. The first `<leader>s` search may take a moment if the index hasn't been built yet.

Embeddings are stored in `.zk/search.db`.
