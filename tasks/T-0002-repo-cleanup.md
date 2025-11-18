---
id: T-0002
title: "Repository cleanup and pruning unused artifacts"
status: planning
priority: medium
owner: ""
created_at: 2025-11-18
updated_at: 2025-11-18
tags: ["cleanup"]
---

## 0. User story / problem

The repository has many README variants and other files that seem unused. You want a cleanup pass to remove redundant docs, unused build scripts, and other cruft so the repo is lean and only keeps what is necessary.

---

## 1. Context gathered by AI(s)

### 1.1 Relevant files / dirs

### 1.2 Summary of current behavior

### 1.3 Risks / constraints / assumptions

### 1.4 Open questions for the human

- Are there any docs (README variants, guides) that must be kept even if they appear redundant?
- Which distribution targets should remain? (e.g., npm, GitHub Packages, others?)
- Any legacy build or publish workflows that should be preserved for compatibility even if not currently used?
- Is there a dependency or file whitelist/blacklist to guide deletions (e.g., keep `/examples`, remove `/scripts/old`)?
- Can we remove CI jobs that are not referenced in the current release flow?
- Should we prioritize reducing repo size (large assets) or focus only on unused scripts/docs?

---

## 2. Proposed success criteria (AI draft)

(AI suggests testable criteria here.)

---

## 3. Approved success criteria (human-edited)

(Human edits this; this becomes the contract.)

---

## 4. Implementation log / notes

- 2025-11-18 – intake: created task and captured open questions

---

## 5. Completion checklist & review

### 5.1 Human review notes

### 5.2 Follow-up tasks (spinoffs)

