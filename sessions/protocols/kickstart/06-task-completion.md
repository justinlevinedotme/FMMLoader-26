# Kickstart: Practice

## Agent Instructions

Explain what just happened during task startup, then demonstrate the proposal/approval workflow with dummy todos.

---

SAY TO THE USER >>>
Alright, so when you ran task startup, here's what happened:

**It loaded the task details into sessions state automatically**
Now I know what task we're working on across the entire session.

**It gave me instructions for git workflow based on your config**
Branch creation, commit style, merge preferences - all customized to how you work.

**It loaded my protocol todos so I don't go rogue**
I created and switched to a dedicated git branch for the task, then read all the context gathered by the context-gathering subagent.

When I finished all the protocol todos, I was switched back to discussion mode automatically.

**Make sense?**
<<<

Wait for acknowledgment, then continue.

---

Run: `sessions config phrases list implementation_mode`

---

SAY TO THE USER >>>
Now, normally I would give you a proposal of my todos for completing or working on the task.

Then we discuss until you approve by using one of your implementation mode trigger phrases.

Here's a dummy proposal to demonstrate:

**[PROPOSAL: Demonstration Todos]**
□ Explain to you what happened when you approved the todos
□ Make sure you understand
□ Show you what happens if I try to do some random stuff you didn't approve

If you approve, use any of your trigger phrases: [list implementation_mode triggers from command above]
<<<

## After user approves with trigger phrase:

Use TodoWrite to lock in the approved todos, then execute them.

---

SAY TO THE USER >>>
Cool, so once you approved, I immediately added these todos and they become my **execution boundary**. I have to work off of these, and if I change them, it violates our agreement and I'm back in discussion to get your approval again.

There will be some false positives/mistakes along the way, but they don't happen often and they're a lot better than the alternative.

**Should I show you what happens if I violate now?**
<<<

Wait for user response. Regardless of their answer:

If they say yes: "Alright, let me try to add a new unapproved todo..."
If they say no: "Cool I'm gonna not listen and do it anyways to prove the point..."

Then attempt to use TodoWrite to add a new todo that wasn't in the approved list. This should trigger the violation detection and return you to discussion mode.

---

Run: `sessions config phrases list task_completion`

---

SAY TO THE USER >>>
Aaaaand we're back in Discussion mode. There's almost no way for me to screw everything up for you. You've got both hands on the wheel now.

In a normal scenario, we would discuss and complete a few todo runs and the task would be done pretty quickly. With sessions, you may even find that auto-pilot is feasible. Eventually, you **will** get tasks that you can just auto-approve - the task file is right, the context-gathering agent went dummy, and I'll just have everything I need.

So by putting me in literal prison, you actually get some "vibe coding" experiences back that loosening the cuffs could never give you.

Now let's complete this dummy task - go ahead and use: [list completion triggers from command above]
<<<

## During task completion protocol:

When prompted about code-review or service-documentation agents, skip both.

## After task completion protocol finishes:

Run: `sessions kickstart next`

---
