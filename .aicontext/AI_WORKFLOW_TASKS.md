# AI Workflow Tasks

This repository uses file-based tasks located in `tasks/T-XXXX-*.md`. Each task uses YAML frontmatter followed by structured sections owned by different agents and humans.

## Status lifecycle

- `planning`
- `needs_context`
- `awaiting_approval`
- `in_progress`
- `in_review`
- `done`

## Task file sections

1. **0. User story / problem** — human-written description of the request or bug.
2. **1. Context gathered by AI(s)** — AI collects relevant code context and summarizes it.
3. **2. Proposed success criteria (AI draft)** — AI proposes testable acceptance criteria.
4. **3. Approved success criteria (human-edited)** — human edits/approves; this is the contract.
5. **4. Implementation log / notes** — AI logs work performed and decisions made.
6. **5. Completion checklist & review** — final notes, review outcomes, and follow-ups.

## Rules

- AIs MUST NOT bypass tasks. Non-trivial work must reference a task.
- Once Section 3 is approved, it is the source of truth for scope.
- Use the sections only as intended; do not repurpose or delete them.
