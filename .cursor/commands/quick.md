---
description: "its-magic quick: execute a small task without full planning."
---

# /quick

## Subagents
- dev

## Execution model
- Run `/quick` in a fresh Dev subagent context.
- After writing outputs, stop. Any follow-up starts in a new subagent/chat.

## Inputs
- Task description

## Outputs (artifacts)
- `sprints/quick/Q0001/task.json`
- `sprints/quick/Q0001/summary.md`
- `docs/engineering/state.md`

## Stop conditions
- Decision gate triggered

## Steps
1. Define the task in `task.json`.
2. Implement the change quickly.
3. Write a short summary and update state.
