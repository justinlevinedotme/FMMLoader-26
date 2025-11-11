# Kickstart: Advanced Features

## Agent Instructions

Briefly introduce advanced features for awareness. Offer to create indexes if desired, explain iterloop pattern, then cover stashed todos.

---

SAY TO THE USER >>>

Alright, a few more things you should know exist:

**Task Indexes**

Think of indexes like project folders - they organize related tasks together. Maybe you have a big feature spanning multiple tasks, or you want to track all the work for a specific subsystem.

Examples:
- Create a 'user-auth' index for login, signup, password reset tasks
- Create a 'performance' index for optimization tasks across services
- Create a 'refactor-api' index for API cleanup work

The system creates and updates indexes automatically at task boundaries (when you complete tasks, start new ones, etc.). But you can also ask me to create a new index anytime it makes sense.

Management is mostly automatic - the protocols handle the bookkeeping. Some people use indexes heavily, others never touch them. Totally up to you.

**Do you want me to create any indexes for you right now?**

<<<

## If user says yes:

Read `sessions/tasks/indexes/INDEX_TEMPLATE.md` and work with the user to fill it out and create appropriate indexes.

## After index creation or if user says no:

---

SAY TO THE USER >>>

**Iterloop Protocol**

Using iterloop is kind of a cheat code - it's a natural language for-loop.

When you say "iterloop" in your message, I automatically get a prompt telling me to present items one at a time and wait for you to say "continue" before moving to the next one.

**Example 1: User provides the list**
```
[[ iterloop ]]
- /api/users endpoint
- /api/posts endpoint
- /api/comments endpoint

1. Present the next endpoint's error handling code
2. Identify any missing try/catch blocks
3. Propose fixes
4. Implement after approval
5. Repeat until finished
```

**Example 2: Claude identifies the list**
```
[After Claude lists all config files in the project]

[[ iterloop ]]
1. Present the next config file
2. Check if environment variables are used properly
3. Discuss any needed changes
4. Update the file
5. Repeat until finished
```

This can be really effective - unreasonably effective. If you find a place to use it, I highly recommend it - it can save you a bunch of time and frustration.

**Any questions about iterloop before we hit the final feature?**

<<<

When answering questions:

- Do not invent information — base answers strictly on content in your current context window.
- If you need additional facts, use read-only tools to consult project docs:
  - Allowed sources:
    - `sessions/*` files (protocols, tasks, hooks, api)
    - `sessions/CLAUDE.sessions.md` (@sessions/CLAUDE.sessions.md)
- If the information is not available, say so explicitly and avoid speculation.
- Avoid jumping ahead — kickstart will cover most questions as you progress.

Wait for user response, then continue.

---

SAY TO THE USER >>>

**Stashed Todos**

If you use context compaction or task creation with active todos, they get stashed automatically. When task creation completes, stashed todos get restored. After running `/clear`, on session start, stashed todos also get restored.

**The only reason you need to know this is so you know it is *always* safe to create tasks when you discover them.**

Example: You're debugging auth and discover a logging bug. Create a task for it immediately - "mek: fix logging race condition in auth flow". Your current work gets stashed, the new task gets created, then your auth todos restore automatically. Zero friction.

That's the whole point. You will discover problems faster than you can solve them - documenting tasks when they become obvious with no friction is a must. So that's why it's such a huge part of the system.

Similarly, it is *always* safe to compact, so don't stress out.

<<<

---

SAY TO THE USER >>>

That's it. Those are the one-off features you should know about.

**Ready to graduate?**

<<<

## After user acknowledges:

Run: `sessions kickstart next`

---
