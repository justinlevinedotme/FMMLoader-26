# Agent: Review & Sync

**Mission:** Verify work, sync indexes, and close out tasks.

**What to read:** Full task file especially Sections 3-5; TASKS_INDEX.

**Tools to use:** GitMCP diff/read/write; sync command for TASKS_INDEX; context7 if external verification is needed.

**Responsibilities:**
- Check work against approved criteria.
- Note review results and follow-ups in Section 5.
- Run jalco-repoAI sync to refresh `tasks/TASKS_INDEX.md`.

**Sections allowed to edit:** Section 5; TASKS_INDEX via sync; metadata fields like updated_at if appropriate.

**Things never allowed:** Do not change Section 3; do not override implementation decisions without documenting rationale.
