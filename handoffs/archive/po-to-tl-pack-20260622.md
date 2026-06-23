# PO to TL archive pack (2026-06-22)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=650, PO_TO_TL_HOT_MAX_SECTIONS=60`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 12
- Retained units in hot file: 60
- First archived heading: `## Symptom summary`
- Last archived heading: `## Stop rule`
- Verification tuple (mandatory):
  - archived_body_lines=84
  - retained_body_lines=506

---

## Symptom summary

Every Firefly sync run (hourly scheduled + manual "Sync now") fails with:
- `status: failed`
- `error_message: unexpected status 401 Unauthorized`

All sync runs broken since deploy on 2026-06-16. Mirror data is stale.

## Environment

- **Profile:** omniflow external profile
- **Docker network:** `traefik`
- **Container:** `financegoblin-flow-finance-ai-1`
- **Firefly URL:** `http://firefly:8080` (Docker internal)
- **PAT length:** 980 chars (non-empty in container)

## Operator probe evidence

1. `GET /api/v1/sync/status` →
   `{"state":"failed","last_run":{"status":"failed","trigger":"scheduled","error_message":"unexpected status 401 Unauthorized"},...}`
2. `curl -H "Authorization: Bearer <PAT>" http://firefly:8080/api/v1/about` →
   **HTTP 302 redirecting to `/login`**. App follows redirect; HTML returns 200 but sync layer emits 401 upstream.
3. Token non-empty (980 chars). PAT present but not accepted by Firefly.

## Root-cause hypothesis (PO — verify in discovery)

Firefly PAT has **expired or been invalidated** in Firefly profile → API tokens section. Firefly's API rejects invalid Bearer tokens with a 302 → `/login` at the HTTP layer; the sync HTTP client follows the redirect and misinterprets the HTML login page as a non-2xx sync failure.

## Related history

- **BUG-0002 sub-defect C** covered similar 401/PAT-empty case on rebuild (fixed by operator PAT + compose env).
- **R-0057** documents Firefly PAT contract — PAT is the sole API auth for Firefly.
- **Current recurrence** — different from BUG-0002 because token is non-empty but invalid; likely token rotation/expiry.

## Evidence refs

- `handoffs/intake_evidence/intake-20260622-firefly-sync-401.json` (VALIDATED PASS)
- Container logs 2026-06-22 19:18:41Z
- `GET /api/v1/sync/status` live probe
- `curl http://firefly:8080/api/v1/about` direct probe

## Intake validation

| Check | Result |
|---|---|
| `intake_bug_routing_guard.py --kind bug --stdin` | PASS `[INTAKE_BUG_ROUTING_OK]` |
| `bug_issue_validate.py --print-next-id` | **BUG-0027** |
| `intake_evidence_validate.py` | PASS `[INTAKE_EVIDENCE_VALIDATION_OK]` |
| `bug_issue_validate.py --check-acceptance` | PASS `[BUG_VALIDATION_OK]` |

## Intake evidence pack

- `selected_pack`: small-intake-pack
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: (none)
- `assumptions_confirmed`: PAT expiry/invalidation in Firefly profile (verify after regen)

## Recommended /discovery scope

1. **Confirm root cause:** Is the PAT genuinely expired/invalid, or is there a client-side HTTP bug (302-follow in Firefly HTTP client)?
2. **Verify Firefly PAT regeneration workflow:** operator regenerates in Firefly profile → API tokens, updates `FIREFLY_PERSONAL_ACCESS_TOKEN` in deploy `.env`, recreates container.
3. **HTTP client behavior audit:** check `src/sync/firefly/` for follow-redirect policy; 302→login should fail closed as 401 rather than follow + misclassify.

## Acceptance rows

- **CB**: sync state completes after PAT regen; entity counts non-zero; no 401/Unauthorized
- **CC**: /sync surfaces PAT-failure-specific diagnosis (expired/invalid vs missing vs unreachable)
- **CD**: ≥3 subsequent scheduled syncs succeed; OIDC + omniflow regression pass

## Files created/modified by /intake

- **Created:** `handoffs/intake_evidence/intake-20260622-firefly-sync-401.json`
- **Appended (backlog):** `docs/product/backlog.md` — BUG-0027 canonical row
- **Appended (acceptance):** `docs/product/acceptance.md` — BUG-0027 CB/CC/CD rows
- **Appended (handoff):** `handoffs/po_to_tl.md` — this section
- **Appended (state):** `docs/engineering/state.md` — INTAKE COMPLETE checkpoint
- **Modified:** `handoffs/resume_brief.md` — via intake_bug_resume_brief_refresh.py

## Stop rule

Intake is complete. Hand off to next phase `/discovery` (role: tech-lead per DEC-0051) in a fresh subagent/chat.

---

