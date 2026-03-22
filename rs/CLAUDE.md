# jk-tools

## Rust dependency APIs
Use the `rustdocs` MCP server (`rustdoc-mcp`) to inspect dependency APIs at the exact pinned version. Don't read source from `target/doc/` or local caches.

## Library context7 IDs
- rmcp: /websites/rs_rmcp

## Validation
When adding, removing, or changing MCP tools, update `VALIDATION.md` to cover the new behavior. Every MCP tool should have at least one validation step, including both happy-path and error cases where relevant.
