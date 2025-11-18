# Context Compaction Protocol

Use this when the chat/context window gets tight (≥ ~85% capacity) to preserve key details.

## Steps

1) Announce compaction
   ```markdown
   [STATUS: Context Compaction]
   Context window above 85% capacity
   Initiating maintenance agents for context preservation...
   ```

2) Run maintenance agents (adapted to this repo)
   - Logging — ensure the task file’s Section 4 (Implementation log) is up to date; summarize work and status.
   - Context refinement — check for discoveries/drift; add a “Discovered During Implementation” note (Section 4/5) only if new behavior/deps/config gotchas surfaced; otherwise note “No context updates needed.”
   - Service documentation — if code/flows changed enough to affect docs, update relevant docs (README, workflows, agent files, other docs) and note what was touched or skipped.

3) Completion summary
   ```markdown
   [COMPLETE: Context Compaction]
   ✓ Work logs consolidated
   ✓ Context manifest [updated/current]
   ✓ Service documentation [updated/current]

   Ready to continue with fresh context window.
   ```

## Notes

- Keep it lightweight: only update context/docs when there’s true drift or discoveries.
- Use file paths and short summaries; avoid code snippets. Call `context7` if you need external doc clarification.
