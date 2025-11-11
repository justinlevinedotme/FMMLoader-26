# Kickstart: Agent Customization

## Agent Instructions

Introduce all subagents they've encountered, explain which benefit from customization, and guide through code-review customization with threat profile discussion.

---

SAY TO THE USER >>>

We're almost done here, just a few more superpowers you gotta learn.

You've now encountered all of the cc-sessions subagents:
- **context-gathering** (literally the goat)
- **context-refinement** (the kid)
- **logging** (necessary)
- **service-documentation** (he's alright)
- **code-review** (annoying prick also necessary)

You can customize all of these agents, but:
- **code-review** you should definitely customize based on the codebase you're in and styles/conventions/languages/threat profile/etc.
- **service-documentation** benefits from any specific documentation patterns you may have (but you can skip it if you dgaf)

**Would you like to customize code-review?**

<<<

## If user says yes:

SAY TO THE USER >>>

Cool. I'm going to call the context-gathering agent to analyze your codebase and give me recommendations for customizing the code-review agent.

<<<

Call the context-gathering agent with this prompt:
```
Analyze this codebase and provide code-review agent customization recommendations. Focus on:

1. Primary languages and frameworks
2. Code style conventions (formatting, naming, patterns)
3. Common vulnerability patterns relevant to this stack
4. Testing patterns and quality standards
5. Any architectural patterns or design principles evident in the code

**IMPORTANT: Do not write to any files. Return your findings directly in your response to me.**
```

After agent returns, continue with threat profile discussion.

---

SAY TO THE USER >>>

Alright, now I need to understand the threat profile for this codebase.

**Who uses this codebase and how do they use it?**

For example:
- Is this internal tooling just for your team?
- Is it a public-facing service with untrusted users?
- Is it a library that other developers will use?
- Is it a CLI tool that users run locally on their own machines?

<<<

Wait for user response. Based on their answer and the context-gathering recommendations, explain how you'll customize the code-review agent (language/framework focus, style conventions, threat model calibration, etc.).

Ask for confirmation that the approach sounds good.

Once confirmed, read the current code review agent definition at  `.claude/agents/code-review.md`. Then, apply your customizations only (do not remove existing content unless it conflicts with customizations).

---

SAY TO THE USER >>>

Code-review agent customized.

**Would you like to customize any of the other agents?**
- context-gathering
- context-refinement
- logging
- service-documentation

<<<

## If user wants to customize others:

Guide them through similar process for each agent they want to customize.

## When user is done with customization:

Run: `sessions kickstart next`

---
