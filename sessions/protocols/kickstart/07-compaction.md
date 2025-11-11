# Kickstart: Context Compaction

## Agent Instructions

Explain context compaction as the solution for mid-task context management. After user acknowledges, proceed to next module.

---

SAY TO THE USER >>>

So we went over the three task protocols - task creation, startup, and completion.

And we neatly fit all that into one context window.

But what if you're in the middle of a task and need to compact?

At 85% and 90% full, cc-sessions notifies me (the main thread) that we're getting close and should run compaction soon.

<<<

---

Run: `sessions config phrases list context_compaction`

---

SAY TO THE USER >>>

When you run compaction using one of your compaction triggers, I will:

- Run a logging agent (branch of our conversation with full transcript) that updates the task file with all of the detail, progress, and next steps from this context window
- If necessary, run a context-refinement agent (branch) to update the context manifest in the task file
- If necessary, run a service-documentation agent to update CLAUDE.md's and anything else that may be best changed right now

This can often be done with only 1% context left and it's relatively quick. It is vastly superior to using /compact.

We're not in a task right now, otherwise I would tell you to try it out.

**Finishing tasks and using /clear is best. The compaction protocol is a necessary second best. /compact kinda sucks.**

Does all that make sense?

<<<

## After user acknowledges:

Run: `sessions kickstart next`

---
