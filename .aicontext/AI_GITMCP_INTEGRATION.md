# AI GitMCP Integration

Use GitMCP tools for all repository interactions. Typical tools (names may vary by client):

- `gitmcp.search` — search the codebase.
- `gitmcp.readFile` — read file contents.
- `gitmcp.writeFile` — write files.
- `gitmcp.applyPatch` — apply diffs/patches safely.
- `gitmcp.listDirectory` — list files and directories.
- `gitmcp.diff` — view changes.

Rules:

- Do not guess file paths; discover with GitMCP.
- Do not bypass the task system; every change maps to a task.
- Use context7 (or similar) for external docs, not GitMCP.

Example (Context agent):

1. List files with `gitmcp.listDirectory`.
2. Search for a keyword with `gitmcp.search`.
3. Read relevant files with `gitmcp.readFile`.
4. Update the task file Section 1 with findings using `gitmcp.writeFile` or `gitmcp.applyPatch`.
