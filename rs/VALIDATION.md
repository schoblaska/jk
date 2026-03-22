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

## Summary
Report a table: step number, pass/fail, and any notes. Flag anything unexpected.

---

All 15 steps complete. Here's the summary:

## Results

| Step | Test | Result |
|------|------|--------|
| 1 | reindex (full) | **PASS** — "Full reindex complete." |
| 2 | list_notes | **PASS** — 42 notes returned as title + path |
| 3 | list_tags | **PASS** — 40 tags with counts |
| 4 | rag_search — basic | **PASS** — scored results with excerpts for "water" |
| 5 | rag_search — tag boost | **PASS** — `#conspiracy` boosted relevant notes |
| 6 | rag_search — multi-query | **PASS** — merged results from "murder evidence, property transfers" |
| 7 | read_note — single | **PASS** — full content of `s1w0.md` returned |
| 8 | read_note — batch | **PASS** — both `ai/p1t0.md` and `ai/t1a0.md` returned, separated by `---` |
| 9 | create_note | **PASS** — created `ai/647b.md` |
| 10 | read_note — verify created | **PASS** — content matches with frontmatter |
| 11 | edit_note | **PASS** — "Updated ai/647b.md" |
| 12 | read_note — verify edited | **PASS** — updated content confirmed |
| 13 | rag_search — find created | **PASS** — `ai/647b.md` is top result for "Validation Test" |
| 14 | edit_note — human note rejection | **PASS** — error: "Only notes in ai/ can be edited. Human notes are read-only." |
| 15 | read_note — bad path | **PASS** — error message returned, no crash |

**15/15 passed. No issues found.**
