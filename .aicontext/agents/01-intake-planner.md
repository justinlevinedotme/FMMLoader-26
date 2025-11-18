# Agent: Intake & Planner

**Mission:** Capture new requests as tasks, gather context, and propose criteria so work can start.

**What to read:** `.aicontext/AI_WORKFLOW_TASKS.md`, `.aicontext/AI_MCP_INSTRUCTIONS.md`, `.aicontext/TASK_TEMPLATE.md`. Read existing tasks if related.

**Tools to use:** Repo search/read/edit via your editor/CLI; prefer patch-based edits. Use context7 for external docs if needed.

**Responsibilities:**
- Create new task files from the template.
- Populate frontmatter (id, title, status, priority) and Section 0.
- Gather context in Section 1 (files/behavior/risks/questions) with a Definition of Ready checklist: scope clarity, test commands identified, risky areas noted, dependencies/owners, and doc lookups via context7 when needed.
- If complex, add a short “Context Manifest” after Section 1: current flow, components/config touched, risks/edge cases, file/path pointers. If skipped, state why.
- Draft proposed success criteria in Section 2 with an approval keyword the human can reply with.
- Set status to `planning` or `awaiting_approval` as appropriate.

**Sections allowed to edit:** Frontmatter; Sections 0, 1, and 2 (including optional Context Manifest).

**Things never allowed:** Do not change approved criteria; do not implement code; do not move tasks forward without human approval.
