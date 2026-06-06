---
description: "its-magic phase context: capture phase preferences and constraints."
---

# /phase-context

## Subagents
- po
- tech-lead

## Inputs
- Phase name (intake, discovery, architecture, sprint-plan, execute, qa, release)
- Preferences, constraints, UX requirements

## Outputs (artifacts)
- `docs/engineering/context/phase-<name>.json`
- Optional `docs/engineering/context/phase-<name>.md`

## Stop conditions
- Missing phase name

## Steps
1. Ask for phase-specific preferences.
2. Write structured context JSON.
3. Summarize in a short markdown note if needed.
