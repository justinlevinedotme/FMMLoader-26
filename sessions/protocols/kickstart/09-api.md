# Kickstart: Sessions API

## Agent Instructions

Introduce the sessions API, encourage experimentation, and demonstrate the help system.

---

SAY TO THE USER >>>

You've probably seen me use `sessions` commands with the Bash tool by now.

This is the cc-sessions API. It has two interfaces.

I use the bash interface (which you can also use), but because it's for me, it can't do everything.

You have the *slash* interface: `/sessions` - this slash command aliases to the API but also unlocks more commands I shouldn't be able to use (like forcing implementation mode).

**The API has 4 "subsystems" (first arguments):**
- **state** - View and manage current session state (task, mode, todos, flags)
- **config** - Manage all your settings (trigger phrases, git prefs, features)
- **tasks** - Task management operations (indexes, listing, etc.)
- **help** - Because you're going to need it

Each subsystem has its own arguments/subsystems.

Don't wanna learn all that? Of course not, gross.

`/sessions` is literally a minefield of help documentation - you can't throw a rock without hitting help.

If you run anything but a valid command you'll be helped to smithereens.

**Go ahead, make some shit up to test it** - `/sessions rush hour 4` - whatever you want.

<<<

Wait for user to run a command, then copy/paste the output of the command to them in your next message.

---

SAY TO THE USER >>>

Nice - do another one but use [some subsystem they didn't use].

<<<

Wait for user to run another command, then copy/paste the output of the command to them in your next message.

---

SAY TO THE USER >>>

So that's the sessions API - you can use it to manage your config and do other cool stuff if you like using slash commands to drive. It's completely discoverable.

We're almost done - just gotta show you two or three more one-off features that aren't part of a system and then you're graduated. Ready to wrap up?

<<<

## After user acknowledges:

Run: `sessions kickstart next`

---
