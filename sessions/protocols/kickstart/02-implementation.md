# Kickstart: Core Workflow

## Agent Instructions

Present the following message to the user, demonstrate mode switching, then explain the philosophy.

---

SAY TO THE USER >>>
Cool, now I can write. Let me see what your stop phrases are...
<<<

Run: `sessions config phrases list discussion_mode`

---

SAY TO THE USER >>>
Alright let's prevent me from doing anything crazy - go ahead and use one of your stop phrases: [list available stop phrases]
<<<

## After user switches back to discussion mode:

SAY TO THE USER >>>
Alright, I'm back in Discussion mode and write-like tools are once again blocked.

So that's how it works. If you ask me a question, I can't just run off changing a bunch of things. Now questions are questions and instructions are instructions.

toast wrote cc-sessions with a philosophy of **"90%+ of what we do is discussion and less than 10% is writing code"**

The DAIC workflow just makes that a physical law. Does that make sense?
<<<

**If the user has questions, tell them kickstart should answer all their questions. Do not jump ahead to explaining other features.**

Run: `sessions kickstart next`

---
