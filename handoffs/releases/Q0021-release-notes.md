# Quick Release Notes — Q0021 / US-0017

**Quick task:** Q0021  
**Story:** US-0017 — README living-doc expansion and troubleshooting  
**Date:** 2026-06-09  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` US-0017 AC-1..AC-5)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS-with-story-scope — `python3 scripts/validate_doc_profile.py --repo . --no-template-parity` exit 0 @ 2026-06-09 release (`[DOC_PROFILE_VALIDATE_OK]`); doc-only scope per S0013 precedent (`DOC_ONLY_SCOPE`)
2. **QA completion gate:** PASS — `sprints/quick/Q0021/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0021/uat.json`, `sprints/quick/Q0021/uat.md`, `sprints/quick/Q0021/verify-work-findings.md`; 5 steps — 5 pass, 0 fail
4. **Isolation compliance gate:** PASS — execute, qa, verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — `runtime-proof-verify-work-20260606-us0017-q0021-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Doc-only DEC-0070 extension: omniflow smoke H3 under Examples, Troubleshooting H3 under Limitations, per-segment Product status maintenance hooks in developer README and runbook, operator user guide US-0017, validator gate PASS.

| Scope | Deliverable |
|-------|-------------|
| **E1** | `### Omniflow smoke (external profile)` — OMNI curls, gate sequence, six-route table |
| **E2** | `### Troubleshooting` — 6-row symptom table; ML-unavailable vs empty-panel distinction |
| **E3** | Product status verify-only at execute; **US-0017 bullet appended at release** |
| **E4** | `docs/developer/README.md` per-segment release/refresh hooks |
| **E5** | Runbook § README maintenance — release segment definition + per-id hooks |
| **UG1** | `docs/user-guides/US-0017.md` operator distill |
| **E6** | `validate_doc_profile --no-template-parity` exit 0 |

**Linked decisions:** DEC-0070 (US-0017 extension)  
**Research fulfilled:** R-0078 (extends R-0066, R-0067)

---

## Run

**Doc-only release:** no application code changes; operator runtime unchanged from prior releases.

- `start_command`: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d` (unchanged; see README Quickstart)
- `runtime_mode`: local | remote (omniflow external profile per US-0010)
- `runtime_context_ref`: `README.md` § Quickstart; `README.md` § Omniflow smoke (external profile); `docs/user-guides/US-0017.md`

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc` (omniflow external) or `http://localhost:8080` (local minimal)
- `service_port`: 8080 (backend SPA proxy)
- `health_endpoint`: `GET /health`

---

## Verify

1. Confirm root `README.md` has `### Omniflow smoke (external profile)` under `## Examples` with OMNI base URL and six-route table.
2. Confirm `### Troubleshooting` under `## Limitations` with 6-row symptom table and ML-unavailable distinction.
3. Confirm `### Product status` includes **US-0017** closure bullet (newest first).
4. Run `python3 scripts/validate_doc_profile.py --repo . --no-template-parity` — expect `[DOC_PROFILE_VALIDATE_OK]` exit 0.
5. Inspect `docs/developer/README.md` Quality gates and runbook § README maintenance for per-segment hooks.
6. **Optional post-release:** operator omniflow smoke curls from README `### Omniflow smoke (external profile)` on `https://financegnome.omniflow.cc`.

**Expected health signal:** doc validator exit 0; `/health` OK when stack running.

---

## Credentials

- Traefik basic auth — operator shell / password manager (placeholder in README curls only)
- `FIREFLY_PERSONAL_ACCESS_TOKEN`, `DATABASE_*` — operator `.env` from `.env.example`; no inline secrets in artifacts

---

## Changes

| Area | Summary |
|------|---------|
| `README.md` | Omniflow smoke H3, Troubleshooting H3, Product status US-0017 bullet |
| `docs/developer/README.md` | Per-segment Product status maintenance wording |
| `docs/engineering/runbook.md` | § README maintenance — release segment + per-id hooks |
| `docs/user-guides/US-0017.md` | Operator user guide (USER_GUIDE_MODE=1) |

---

## Known Issues

- Optional omniflow live smoke curls documented but not required for doc-only closure
- `scripts/check-user-visible-metadata.py` absent — US-0071 guard skipped (repo precedent)
- Pre-existing `TEST_COMMAND` (`bash tests/run-tests.sh`) failures out of US-0017 doc scope

---

## Queue linkage

- Queue row: `handoffs/release_queue.md` — Q0021 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.17.0-us0017`

## Milestone

**US-0017 released** — README living-doc expansion with omniflow smoke, troubleshooting, and enforced upkeep hooks.
