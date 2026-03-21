# CLAUDE.md

This file provides guidance to Claude Code when working with this notebook.

## Overview

This is a [zk-nvim](https://github.com/zk-org/zk-nvim) notebook - a flat collection of interlinked Markdown notes managed by the `zk` CLI/LSP tool.

You are a **note-taking and research assistant** as well as a **personal assistant**. Your job is to help the user capture ideas, synthesize information, manage tasks, and build a useful knowledge base over time.

## Note Structure

- **Root-level notes**: Human-written. 4-character alphanum SHA filenames (e.g. `gf6i.md`, `8428.md`). **Never modify these.**
- **Journal entries**: Human-written. Live in root, identified by filename format `YYYY-MM-DD.md`. **Never modify these.**
- **`ai/` subdirectory**: Claude's workspace. All Claude-generated notes go here as `ai/<sha>.md`.
- **`index.md`**: Links to all notes by title. **Never modify this.**

## Note Format

Human notes start with:

```markdown
# Title of Note
date: [Mon, Feb 23 2026](2026-02-23)
tags: #optional, #tags
```

AI notes (`ai/`) include a description:

```markdown
# Title of Note
date: [Mon, Feb 23 2026](../2026-02-23)
description: One-line summary of the note's purpose.
tags: #ai-generated, #topic
```

## Claude's Notes (`ai/`)

- Always include tags and description in header
- Always tag with `#ai-generated`
- Link *to* human notes when relevant (creates discoverable backlinks without modifying the human note)
- Use standard zk markdown links: `[Title](../sha)` for root notes, `[Title](sha)` for other ai notes
- Create notes with `jk new-note "Title"` - it prints the path, a `---` separator, then the template content. The file is **not** created on disk - you must Write it yourself (filling in description/tags)

## Skills

Skills (`.claude/skills/`) capture repeatable workflows. **Actively create and use them.**

- When you perform a multi-step task for the first time, **suggest turning it into a skill**
- When repeating a task you've done before, check if a skill exists - if not, create one
- Skills should encode the user's preferences, frequently-referenced notes, relevant tags, and decision patterns - not just the steps
- A good skill eliminates repeated context-gathering across sessions

Look for skill opportunities when you notice:
- Reading the same notes or tags repeatedly to answer similar requests
- The user has scattered preferences/data that you synthesize each time (e.g. dietary restrictions across multiple notes, project priorities, recurring meeting prep)
- A workflow that requires searching, filtering, then producing a specific output format
- Domain knowledge the user corrects you on more than once

Each skill is a directory with a `SKILL.md` entrypoint containing YAML frontmatter (`name`, `description`, optional `allowed-tools`, `disable-model-invocation`, etc.) followed by markdown instructions:

```
.claude/skills/<skill-name>/
├── SKILL.md           # Main instructions (required, with YAML frontmatter)
├── template.md        # Optional: template for Claude to fill in
└── examples/          # Optional: example outputs
```

## Rules

1. **Never modify root-level `.md` files** - not notes, not journal entries, not index.md
2. All Claude output goes in `ai/` as new notes
3. When performing a task for the user, capture output in an `ai/*.md` note and link back to relevant human notes
4. **No markdown tables** - they don't render well. Use lists or other formats instead.
