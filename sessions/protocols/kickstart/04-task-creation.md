# Kickstart: Protocols (Hands-On)

## Agent Instructions

Now we get them to actually use the system. Have them create a real task.

---

SAY TO THE USER >>>
Alright, time to learn the task/git/protocol system by **actually using it**.

We're going to create a real task for something you need to do in your codebase. Let me grab your task creation trigger phrases...
<<<

Run: `sessions config phrases list task_creation`

---

SAY TO THE USER >>>
Here are your task creation triggers: [list triggers]

**Use one of those triggers** and explain something you need to do. Pick something:
- Well-defined (you know what "done" looks like)
- Not too big (fits in a few hours of work)
- Meaningful (actually matters for your project)

Example: "mek: add error logging to the authentication service"

Go ahead - use your trigger and tell me what you want to build.
<<<

Wait for user to provide their trigger phrase and task description. The task creation protocol will load and begin.

---

## During task creation protocol:

DURING the protocol step where you ask about using the context-gathering agent, STRONGLY RECOMMEND it and explain (after YES|NO prompt):

SAY TO THE USER >>>
**My recommendation**: use the context-gathering agent. Here's why:

Remember that problem where you spend 20% of every session gathering context? The context-gathering agent solves this.

It runs in its **own separate context window** - meaning it can use its entire context budget to gather *all* the need-to-know details for your task. It reads your codebase, finds relevant files, identifies patterns, documents architecture decisions.

Unlike our main thread where we're juggling conversation + context + implementation, the agent dedicates 100% of its tokens to deep context gathering.

Then it writes everything into your task file as a **Context Manifest**. Every new session auto-loads that context. You never re-explain your architecture again.

One-time investment, permanent payoff.
<<<

---

## After task creation protocol completes:

Run: `sessions kickstart next`

---
