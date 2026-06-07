# Design Concept — US-0016

## Summary

US-0016 delivers a **root `README.md`** as the operator and contributor entry point for Flow Finance AI — a DEC-0059 split-layout living document validated by `scripts/validate_doc_profile.py`. The "UX" is information architecture: scannable H2 structure, a capped **Product status** subsection, and deep links to runbook, user guides, and developer shard — not a UI surface.

## Goals

- Root README with five required user H2s + `## Contributing` pointer under profile `(both, balanced)`
- Flow Finance AI-specific Purpose, Quickstart (compose profiles), Examples, Limitations, Related documentation
- Living **Product status** (8 bullets max) updated at release and refresh-context when US/BUG closes
- Validator PASS at release gate with `--no-template-parity` until `template/` tree ships (**DEC-0070**)
- Runbook § README maintenance documenting phase-boundary hooks

## Non-goals

- Per-commit README automation or backlog dump in root
- Partial `template/README.md` stub without full dev shard mirror
- Replacing `docs/user-guides/US-xxxx.md` or spec-pack per-story artifacts
- Full its-magic framework manual

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0070 | `--no-template-parity` until full `template/` | R-0067 §1; reject half-stub |
| DEC-0070 | `### Product status` under `## Purpose` | Budget-safe; scannable; R-0067 §2 |
| DEC-0070 | Release + refresh-context hooks only | R-0066; rejects per-commit churn |
| DEC-0059 (doc profile) | Split layout unchanged | US-0077; dev depth in `docs/developer/README.md` |

**References:** [R-0066](research.md#r-0066--root-readme-split-layout-and-living-doc-maintenance), [R-0067](research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks)
