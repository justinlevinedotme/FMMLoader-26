# Kickstart: Tasks

## Agent Instructions

Present the following message to the user explaining tasks as work units.

---

SAY TO THE USER >>>
Alright, System 2: **Task Management**

Here's what you're juggling when working with a coding agent:
- What work you're doing
- How you save it (git workflows)
- How you define it
- How you track progress
- How you complete and archive it

Over-organizing this stuff becomes harder than it's helpful. Workflows become religion (looking at you, agile/scrum).

**Tasks are simple. Everybody gets tasks. Have you heard of tasks before?**
<<<

Wait for user response, acknowledge briefly, then continue.

---

SAY TO THE USER >>>
cc-sessions uses tasks as a **session/context boundary**.

Here's why: Claude doesn't need to know your entire codebase. He needs to know **the slice required to do one thing the right way.** That's it.

A task is a file that tracks work from "I want to do this" to "it's done and I never think about it again."

Tasks solve multiple problems at once - session persistence, context management, git workflows, progress tracking. One concept, multiple wins.

**Does that click?**
<<<

Wait for user response, acknowledge briefly, then continue.

---

SAY TO THE USER >>>
What happens with tasks:
- Work gets logged in the file
- Full context gets built automatically
- Task loads as first context in new chats
- Survives compaction/clearing

**Tasks ARE what you're doing** when you use cc-sessions (mostly).

You can work without a task - good for quick stuff. But mostly? Tasks.
<<<

Run: `sessions kickstart next`

---
