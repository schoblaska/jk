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

## 13. patch_note — happy path
- Call `patch_note` on the note from step 9: set `old_content` to a unique line from the body, `new_content` to replacement text
- Expect: success message "Patched ai/..."

## 14. read_note — verify patched
- Read the note from step 9
- Expect: the replaced text appears, original text is gone, rest of note intact

## 15. patch_note — not found
- Call `patch_note` on the note from step 9 with `old_content` set to text that doesn't exist in the note
- Expect: error "old_content not found in the note"

## 16. patch_note — ambiguous match
- Call `patch_note` with `old_content` set to a very short string that appears multiple times (e.g. a single common word)
- Expect: error mentioning the match count and suggesting more context

## 17. patch_note — human note rejection
- Pick a root-level note path (not in `ai/`). Call `patch_note` on it
- Expect: error saying only ai/ notes can be edited

## 18. rag_search — find created note
- Call `rag_search` with query `"Validation Test"`
- Expect: the note from step 9 appears in results

## 19. edit_note — human note rejection
- Pick a root-level note path (not in `ai/`). Call `edit_note` on it
- Expect: error saying only ai/ notes can be edited

## 20. read_note — bad path
- Call `read_note` with `paths: ["nonexistent/file.md"]`
- Expect: error message (not a crash)

## 21. recent_journals — user journals
- Call `recent_journals` with default params (or `source: "user"`)
- Expect: full content of recent user journal entries (YYYY-MM-DD.md files), reverse chronological order

## 22. recent_journals — with limit
- Call `recent_journals` with `limit: 2`
- Expect: exactly 2 entries

## 23. append_ai_journal — first call (creates file)
- Call `append_ai_journal` with content `"## Session Start\nValidation test session began."`
- Expect: returns path like `ai/YYYY-MM-DD.md` (today's date)

## 24. read_note — verify AI journal created
- Call `read_note` with the path from step 23
- Expect: frontmatter with today's date, `#ai-journal` tag, and the appended content

## 25. append_ai_journal — second call (appends)
- Call `append_ai_journal` with content `"## Update\nSecond entry appended."`
- Expect: returns same path as step 23

## 26. read_note — verify append
- Read the AI journal again
- Expect: both entries present — "Session Start" and "Update" sections

## 27. recent_journals — AI journals
- Call `recent_journals` with `source: "ai"`
- Expect: the AI journal from step 23 appears in results

## 28. recent_journals — all journals
- Call `recent_journals` with `source: "all"`
- Expect: both user and AI journal entries, interleaved by date

## 29. rag_search — find AI journal
- Call `rag_search` with query `"#ai-journal"`
- Expect: the AI journal from step 23 appears in results

## Summary
Report a table: step number, pass/fail, and any notes. Flag anything unexpected.

---



**Step 29: PASS** — AI journal `ai/2026-03-21.md` appears as top result [0.43] for `#ai-journal` query.

---

## Summary

| Step | Test | Result | Notes |
|------|------|--------|-------|
| 1 | reindex | PASS | Full reindex completed |
| 2 | list_notes | PASS | TSV output, 42 notes |
| 3 | list_tags | PASS | 40 tags with counts |
| 4 | rag_search — basic | PASS | Scored results with paths, excerpts |
| 5 | rag_search — tag boost | PASS | `#investigation` notes boosted |
| 6 | rag_search — multi-query | PASS | Merged results from comma-separated queries |
| 7 | read_note — single | PASS | Full markdown returned |
| 8 | read_note — batch | PASS | Two notes separated by `---` |
| 9 | create_note | PASS | Returned `ai/18d2.md` |
| 10 | read_note — verify created | PASS | Content + frontmatter correct |
| 11 | edit_note | PASS | Overwrite succeeded |
| 12 | read_note — verify edited | PASS | Updated content confirmed |
| 13 | patch_note — happy path | PASS | "Patched ai/18d2.md" |
| 14 | read_note — verify patched | PASS | Replacement correct, rest intact |
| 15 | patch_note — not found | PASS | Error: "old_content not found in the note" |
| 16 | patch_note — ambiguous | PASS | Error: "old_content appears 2 times" |
| 17 | patch_note — human note | PASS | Error: "Only notes in ai/ can be edited" |
| 18 | rag_search — find created | PASS | Validation Test is top result |
| 19 | edit_note — human note | PASS | Error: "Only notes in ai/ can be edited" |
| 20 | read_note — bad path | PASS | Error: "No such file or directory", no crash |
| 21 | recent_journals — user | PASS | 7 user journals, reverse chronological |
| 22 | recent_journals — limit | PASS | Exactly 2 entries returned |
| 23 | append_ai_journal — create | PASS | Returned `ai/2026-03-21.md` |
| 24 | read_note — verify journal | PASS | Frontmatter + `#ai-journal` tag + content |
| 25 | append_ai_journal — append | PASS | Same path returned |
| 26 | read_note — verify append | PASS | Both sections present |
| 27 | recent_journals — AI | PASS | AI journal appears |
| 28 | recent_journals — all | PASS | User + AI journals interleaved by date |
| 29 | rag_search — find journal | PASS | AI journal is top result for `#ai-journal` |

**29/29 PASS. All tools functioning correctly, all error cases handled gracefully.**
