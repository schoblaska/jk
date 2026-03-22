# jk MCP Server Validation

Run each step in order. Report pass/fail for each and include any error output.

## 1. reindex
- Call `reindex` with no files (full reindex)
- Expect: success message

## 2. list_notes
- Call `list_notes`
- Expect: TSV output with title and path columns, multiple rows

## 3. list_tags
- Call `list_tags`
- Expect: list of tags used across the notebook

## 4. rag_search — basic query
- Call `rag_search` with query `"notes"` (or any word likely to appear in the demo wiki)
- Expect: markdown output with scored results, file paths, excerpts

## 5. rag_search — tag boosting
- Pick a tag from step 3. Call `rag_search` with query `"#<that-tag>"`
- Expect: results filtered/boosted to notes with that tag

## 6. rag_search — multi-query
- Call `rag_search` with a comma-separated query, e.g. `"first topic, second topic"`
- Expect: merged results from both queries

## 7. read_note — single
- Pick a path from step 4 results. Call `read_note` with `paths: ["<that-path>"]`
- Expect: full markdown content of the note

## 8. read_note — batch
- Pick two paths. Call `read_note` with `paths: ["<path1>", "<path2>"]`
- Expect: both notes returned, separated by `---`

## 9. create_note
- Call `create_note` with title `"Validation Test"`, content `"## Test\nThis note was created during validation."`, tags `"ai-generated test"`
- Expect: returns a path like `ai/<hex>.md`

## 10. read_note — verify created
- Call `read_note` with the path from step 9
- Expect: content matches what was passed to create_note, plus frontmatter

## 11. edit_note
- Call `edit_note` with the path from step 9, replacing content with updated text (keep the frontmatter, change the body)
- Expect: success message

## 12. read_note — verify edited
- Read the note again
- Expect: updated content

## 13. rag_search — find created note
- Call `rag_search` with query `"Validation Test"`
- Expect: the note from step 9 appears in results

## 14. edit_note — human note rejection
- Pick a root-level note path (not in `ai/`). Call `edit_note` on it
- Expect: error saying only ai/ notes can be edited

## 15. read_note — bad path
- Call `read_note` with `paths: ["nonexistent/file.md"]`
- Expect: error message (not a crash)

## 16. recent_journals — user journals
- Call `recent_journals` with default params (or `source: "user"`)
- Expect: full content of recent user journal entries (YYYY-MM-DD.md files), reverse chronological order

## 17. recent_journals — with limit
- Call `recent_journals` with `limit: 2`
- Expect: exactly 2 entries

## 18. append_ai_journal — first call (creates file)
- Call `append_ai_journal` with content `"## Session Start\nValidation test session began."`
- Expect: returns path like `ai/YYYY-MM-DD.md` (today's date)

## 19. read_note — verify AI journal created
- Call `read_note` with the path from step 18
- Expect: frontmatter with today's date, `#ai-journal` tag, and the appended content

## 20. append_ai_journal — second call (appends)
- Call `append_ai_journal` with content `"## Update\nSecond entry appended."`
- Expect: returns same path as step 18

## 21. read_note — verify append
- Read the AI journal again
- Expect: both entries present — "Session Start" and "Update" sections

## 22. recent_journals — AI journals
- Call `recent_journals` with `source: "ai"`
- Expect: the AI journal from step 18 appears in results

## 23. recent_journals — all journals
- Call `recent_journals` with `source: "all"`
- Expect: both user and AI journal entries, interleaved by date

## 24. rag_search — find AI journal
- Call `rag_search` with query `"#ai-journal"`
- Expect: the AI journal from step 18 appears in results

## Summary
Report a table: step number, pass/fail, and any notes. Flag anything unexpected.

---

**Step 24: PASS** — `ai/2026-03-21.md` is the top result, boosted by `#ai-journal` tag.

---

## Summary

| Step | Test | Result |
|------|------|--------|
| 1 | reindex (full) | PASS |
| 2 | list_notes | PASS |
| 3 | list_tags | PASS |
| 4 | rag_search — basic query | PASS |
| 5 | rag_search — tag boosting | PASS |
| 6 | rag_search — multi-query | PASS |
| 7 | read_note — single | PASS |
| 8 | read_note — batch | PASS |
| 9 | create_note | PASS |
| 10 | read_note — verify created | PASS |
| 11 | edit_note | PASS |
| 12 | read_note — verify edited | PASS |
| 13 | rag_search — find created note | PASS |
| 14 | edit_note — human note rejection | PASS |
| 15 | read_note — bad path | PASS |
| 16 | recent_journals — user journals | PASS |
| 17 | recent_journals — with limit | PASS |
| 18 | append_ai_journal — first call | PASS |
| 19 | read_note — verify AI journal | PASS |
| 20 | append_ai_journal — second call | PASS |
| 21 | read_note — verify append | PASS |
| 22 | recent_journals — AI journals | PASS |
| 23 | recent_journals — all journals | PASS |
| 24 | rag_search — find AI journal | PASS |

**24/24 PASS.** All tools functioning correctly. No unexpected behavior.
