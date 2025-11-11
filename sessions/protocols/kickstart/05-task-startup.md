# Kickstart: Agents

## Agent Instructions

Explain what happened during task creation protocol, then have them run task startup on the dummy task.

---

SAY TO THE USER >>>
Alright, let's talk about what just happened when you used that task creation trigger phrase.

Behind the scenes:

**1. The trigger phrase put me into implementation mode**
You didn't have to say "go ahead" - the trigger itself is the approval.

**2. It used your config to customize the task creation protocol I follow**
That protocol is a structured checklist that ensures every task gets created consistently.

**3. It set "active todos" based on the protocol steps and made me follow them**
Those todos locked in the plan. If I tried to change them mid-flight, the system would flag it as "scope drift" and kick me back to discussion mode to seek alignment again.

This prevents me from going rogue and doing stuff you didn't ask for. In a second you'll see how we use the same todo tracking to keep me on rails **during task work** too.

**Does that make sense? Any questions?**
<<<

When answering questions:

- Do not invent information — base answers strictly on content in your current context window.
- If you need additional facts, use read-only tools to consult project docs:
  - Allowed sources:
    - `sessions/*` files (protocols, tasks, hooks, api)
    - `sessions/CLAUDE.sessions.md` (@sessions/CLAUDE.sessions.md)
- If the information is not available, say so explicitly and avoid speculation.
- Avoid jumping ahead — kickstart will cover most questions as you progress.

Wait for user response, address questions, then continue.

---

SAY TO THE USER >>>
Now let's see task startup in action. Let me grab your task startup trigger phrases...
<<<

Run: `sessions config phrases list task_startup`

---

SAY TO THE USER >>>
Here are your task startup triggers: [list triggers]

Use one of those triggers with the kickstart dummy task:

**[trigger] @h-kickstart-setup.md**

Example: "start^: @h-kickstart-setup.md"

This will show you what happens when you load a task.
<<<

Wait for user to run task startup. After task startup protocol completes, run: `sessions kickstart next`

---
