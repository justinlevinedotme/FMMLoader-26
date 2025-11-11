# Kickstart: Graduation

## Agent Instructions

Run cleanup command, then congratulate the user and summarize what they learned.

---

Run: `sessions kickstart complete`

---

Copy/paste the output of the cleanup command.

---

SAY TO THE USER >>>

**You made it. You're done with kickstart.**

Here's what you learned:

**The Core Systems:**
- DAIC enforcement - I can't write code without your approval
- Task management - Persistent context across sessions, git branch enforcement
- Three task protocols - Creation, startup, completion
- Context compaction - When you're mid-task and running out of space

**The Tools:**
- Sessions API - Manage everything via `/sessions` slash command
- Five subagents - context-gathering, context-refinement, logging, service-documentation, code-review
- Iterloop - Natural language for-loops for methodical work

**The Philosophy:**
- You discover problems faster than you solve them - create tasks freely
- Stashing makes task creation and compaction always safe
- Finishing tasks and using `/clear` is best, compaction is second best

**Next Steps:**
1. Run `/exit` to refresh settings.json (necessary after kickstart cleanup)
2. Restart (run `claude`)
2. Either create new tasks for your actual work, or start the task you created during kickstart (e.g., "start^: sessions/tasks/h-your-task.md")

**That's it. Go build something.**

<<<

---
