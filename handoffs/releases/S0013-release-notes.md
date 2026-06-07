# Sprint Release Notes — S0013

**Sprint:** S0013  
**Date:** 2026-06-08  
**Stories:** US-0016  
**Queue status:** released  
**Orchestrator:** `auto-20260606-us0016-001`  
**Decision:** DEC-0070

---

## Gate results

1. **Check-in test gate:** PASS-with-story-scope — release re-ran `python3 scripts/validate_doc_profile.py --repo . --no-template-parity` @ 2026-06-08 (exit 0); full `TEST_COMMAND` pre-existing fail on `wealth_uses_in_app_portfolio_analytics` (informational QA-001; out of US-0016 doc scope)
2. **QA completion gate:** PASS — `sprints/S0013/qa-findings.md` (AC-1–AC-5 verified; 0 blocking findings)
3. **UAT completeness gate:** PASS — `sprints/S0013/uat.json`, `sprints/S0013/uat.md` (6/6; AC-6 deferred vacuous)
4. **Isolation compliance gate:** PASS — intake through verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — verify-work tuple `runtime-proof-verify-work-20260608-us0016-s0013-001`
6. **Release finalization gate:** PASS

**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Run

- `start_command`: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai`
- `runtime_mode`: local | remote (omniflow external profile per US-0010)
- `runtime_context_ref`: `docs/engineering/runbook.md` § Omniflow external deploy; `README.md` § Quickstart (minimal / bundled-firefly / external profiles)

**Doc-only release:** no application code changes in US-0016; operator runtime unchanged from prior releases.

## Connect

- `service_url`: `https://financegnome.omniflow.cc` (omniflow external) or `http://localhost:8080` (local minimal)
- `service_port`: 8080 (backend); 3000 (frontend dev)
- `health_endpoint`: `/health`

## Verify

- `verification_steps`:
  1. Clone repo; confirm root `README.md` exists with DEC-0059 user H2 sections (Purpose, Quickstart, Examples, Limitations, Related documentation).
  2. Run `python3 scripts/validate_doc_profile.py --repo . --no-template-parity` — expect `[DOC_PROFILE_VALIDATE_OK]` exit 0.
  3. Confirm `## Contributing` links to `docs/developer/README.md`; no `## DEV_*` H2 in root README.
  4. Confirm `### Product status` under `## Purpose` includes US-0016 closure bullet.
  5. Inspect runbook § README maintenance (US-0016) for release/refresh-context hooks.
  6. Deploy per Quickstart profile table if validating runtime (unchanged from US-0010).
- `expected_health_signal`: doc validator exit 0; `/health` OK when stack running

## Credentials

- `credential_source_refs` (env names only):
  - Compose: `COMPOSE_FILE`, `COMPOSE_PROFILES`, `DATABASE_*`, `FIREFLY_PERSONAL_ACCESS_TOKEN`, `TRAEFIK_HOST`
- `expected_value_source`: operator `.env` at repo root (from `.env.example`); no secrets in README

## Known Issues

- `TEST_COMMAND` (`bash tests/run-tests.sh`) fails on pre-existing `wealth_uses_in_app_portfolio_analytics` regression (QA-001; not US-0016).
- `scripts/check-user-visible-metadata.py` absent — US-0071 guard skipped (QA-002).
- AC-6 (`template/README.md` parity) vacuous until `template/` tree lands.

## Deliverables (US-0016)

- Root `README.md` — DEC-0059 split layout with Flow Finance AI content (T-0137–T-0140)
- `tests/run-tests.sh` — doc-profile CI gate (T-0141)
- `docs/engineering/runbook.md` § README maintenance (T-0142)
- `docs/developer/README.md` — quality gates pointer (T-0143)
- `installer.py` — scratchpad merge support for validator (T-0141 blocker fix)
- Decision: DEC-0070

## Queue linkage

- Queue row: `handoffs/release_queue.md` — S0013 `status=released`
- `release_notes_ref`: this file
- `release_version`: `0.13.0-us0016`

## Milestone

**US-0016 released** — root README living documentation for operators and contributors.
