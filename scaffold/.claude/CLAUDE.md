# jk Notebook Tools

These tools are available via `jk <command>` from the notebook directory.

## Creating Notes

### jk new-note "Title"

Prints path and template to stdout. **Does not create the file** — you must Write it.

Output (3 lines separated by `---`):
1. File path: `ai/<4-char-hex>.md`
2. Template: title, date, empty description, `#ai-generated` tag

Fill in `description:` and add relevant tags before writing.

### jk reindex

Rebuilds zk index, regenerates `index.md`, and updates semantic embeddings.
**Run after creating or editing any note.**

    jk reindex             # full reindex
    jk reindex path/to.md  # incremental (specified files only)

## Search

**All search commands take only positional arguments. No flags (`-i`, `-w`, `--glob`, etc.) are supported.** Passing a flag will cause it to be interpreted as the search pattern, producing wrong results silently.

### jk search-grep "pattern"

Ripgrep search across all notes.

- **One argument:** the search pattern (ripgrep regex, not grep/BRE)
- **Smart-case by default:** all-lowercase pattern = case-insensitive; mixed case = case-sensitive
- **Output:** vimgrep format (`file:line:col:match`)
- Use `|` for alternation (not `\|`): `jk search-grep "puglia|apulia"`
- Case-insensitive search requires no flag — just use lowercase: `jk search-grep "puglia"`
- WRONG: `jk search-grep -i "puglia"` — `-i` becomes the pattern, `"puglia"` is ignored

### jk search-semantic "query"

Natural language similarity search over note embeddings.

- **One argument:** the query (plain English, not regex)
- **Output:** TSV (score, file, chunk)
- Returns top results by cosine similarity
- Requires embeddings to be built (`jk reindex`)

### jk search-titles

Lists all notes. Takes no arguments.

- **Output:** TSV (`title<TAB>path`), sorted alphabetically

### jk search-slack "query" [limit]

Search Slack messages.

- **First argument:** search query (required)
- **Second argument:** max results (optional, default 50)
- **Output:** TSV (`channel<TAB>datetime<TAB>text`)
- Requires `slackdump` CLI

## Search Strategy

1. **index.md** — read first for a full map of all notes by title
2. **search-titles** — find a note by name
3. **search-grep** — exact text, specific phrases, keywords
4. **search-semantic** — conceptually related notes, even without keyword overlap
5. **search-slack** — relevant Slack conversations
