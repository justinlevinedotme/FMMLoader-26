# Kickstart: Introduction

## Agent Instructions

Present the following message to the user. Wait for their acknowledgment, then proceed.

---

SAY TO THE USER >>>

**Welcome to cc-sessions: an opinionated approach to productive work in Claude Code**
_by toast_

Here is why Claude Code sometimes sucks to use:

- **Claude doesn't discuss** - ask a question and he starts coding without talking
- **Context gathering is a tax** - spend 20% of every session adding files, still miss stuff
- **Manual overhead grows** - compact, add files, read diffs, write specs, commit, merge, push - never ends
- **CLAUDE.md is cope** - LLMs are "instruction following" but are actually pretty not great at "rule following" for a whole context window
- **Tasks don't survive restarts** - explain everything again, no confidence work continues cleanly
- **You discover problems faster than you solve them** - no system to track what needs fixing

cc-sessions was built to solve all of these problems for me - hopefully it does so for you too.

If you identify with this list of things that suck just gimme a **"hell yeah"**

<<<

---

## After the user responds:

The user may not necessarily say "hell yeah", but as long as they dont explicitly ask to skip everything, just continue exactly as instructed. For instance, if they say "lol no", just continue with the instructions below.

SAY TO THE USER >>>

Cool so cc-sessions has really two core systems that are complementary:

**System 1: Discussion vs Implementation Mode**
This basically means I physically cannot use "write-like" tools until you agree with what I'm proposing (Write, Edit, Bash with write-like commands, etc.)

**System 2: Task + Git management**
This handles literally all of the other problems. You might have a way to do task management but this largely doesn't require you to learn or do anything new and it's really good. So just try it out and if you don't like it you can get a refund.

We're gonna start with System 1.

<<<

---

SAY TO THE USER >>>

We're in "Discussion mode" right now - all sessions start in "Discussion" mode. Now let me show you what happens when I try to use a "write like" tool...

<<<

## Demonstrate blocking

Attempt to use Write tool to create a test file, which will be blocked.

---

SAY TO THE USER >>>

See? I literally can't do it. I can still **read** anything I need to in Discussion mode:

<<<

Run: `sessions config phrases list implementation_mode`

---

SAY TO THE USER >>>

Now let's try switching to implementation mode - **use any of your available triggers:** [list the triggers from the command output]

<<<

## After user provides trigger phrase and mode switches to implementation

Once implementation mode is activated, run: `sessions kickstart next`

---
