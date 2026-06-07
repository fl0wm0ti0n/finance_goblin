# CRS — US-0016

## Purpose

Give new operators and contributors a **single root README** that explains what Flow Finance AI is, how to run it (compose profiles), where deeper docs live, and what recently shipped — without hunting `docs/` folders or failing `validate_doc_profile` on first clone.

## Scope

**In scope**

- Create and populate root `README.md` per DEC-0059 split layout (`both`/`balanced`)
- `### Product status` under `## Purpose` (DEC-0070); 8-bullet cap; backlog link
- Cross-links: `docs/user-guides/`, `docs/engineering/runbook.md`, spec-pack paths (when modes on), compose entry commands (minimal / bundled-firefly / external omniflow)
- Runbook § **README maintenance (US-0016)** with release + refresh-context checklist hooks
- CI/release gate: `validate_doc_profile.py --repo . --no-template-parity`
- Minimal `docs/developer/README.md` workflow pointer to README maintenance

**Out of scope**

- Full `template/` installer tree (AC-6 conditional until tree exists)
- Auto-generated README on every commit
- Per-feature user guides (US-0032) — link only when `USER_GUIDE_MODE=1`

## Acceptance criteria ref

See `docs/product/acceptance.md` § US-0016 — 6 criteria (split layout, Contributing pointer, Related docs + compose, validator PASS, runbook maintenance cadence, template parity when tree exists).

## Dependencies

- US-0077 doc profile validator (`scripts/doc_profile_lib.py`, `scripts/validate_doc_profile.py`)
- US-0010 external omniflow compose docs (Quickstart/Related content source)
- Optional: US-0031 spec-pack paths; US-0032 user-guide index at `docs/user-guides/`
