# AI MCP Instructions

## If GitMCP tools are not available

If you cannot see any GitMCP resources or tools:

1. Fall back to reading files directly from the repo using the client’s native “open file” / “search in workspace” capabilities.
2. Manually open and read, in this order:
   - `.aicontext/AI_WORKFLOW_TASKS.md`
   - `.aicontext/AI_MCP_INSTRUCTIONS.md`
   - `.aicontext/AI_GITMCP_INTEGRATION.md`
   - `.aicontext/CODESTYLE.md`
   - The relevant agent file in `.aicontext/agents/`.
3. When you need to search for code:
   - Use the editor’s built-in search (or “Search in project”) instead of GitMCP.
4. When you need to edit files:
   - Edit them directly in the workspace instead of using `gitmcp.writeFile` or `gitmcp.applyPatch`.
5. Continue to follow the jalco-repoAI task lifecycle exactly; only the transport layer (GitMCP vs direct FS) changes.


Use MCP tools to operate inside this repository. Follow these guidelines:

- **GitMCP** handles repository operations:
  - search code
  - list files and directories
  - read files
  - write files
  - apply patches/diffs
  - view changes/diffs
- **context7** (or similar) provides external documentation lookups for frameworks, libraries, and APIs. Do not mix external lookups with repository edit tools.

## Workflow for tasks

1. **Intake**: create a task from the template in `tasks/`.
2. **Context**: use GitMCP search/read tools to fill Section 1.
3. **Criteria**: draft proposed success criteria in Section 2.
4. **Approval**: a human edits Section 3 and sets status to `in_progress`.
5. **Implementation**: use GitMCP to edit code; log work in Section 4.
6. **Docs/Cleanup**: update docs/comments/tests as needed.
7. **Review**: human verifies and sets status to `done`.
