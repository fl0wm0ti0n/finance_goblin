# PO to TL archive pack (2026-06-22)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=650, PO_TO_TL_HOT_MAX_SECTIONS=60`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 18
- Retained units in hot file: 60
- First archived heading: `## Research summary`
- Last archived heading: `## Recommended next phase`
- Verification tuple (mandatory):
  - archived_body_lines=147
  - retained_body_lines=506

---

## Research summary

R-0099 extended §10 (research phase findings). **Critical correction to discovery hypothesis:** Firefly **does not return 302** for API calls with `Accept: application/json` — it returns **401 JSON** directly. The 302→/login behavior observed in discovery curl probes was because those probes lacked the `Accept: application/json` header. The app **always** sends this header at `firefly/mod.rs` L133, so it receives 401 directly. Reqwest redirect following is irrelevant (no redirect happens for JSON-accept requests).

## Frozen gates (research → architecture carry)

| Gate | Decision |
|------|----------|
| **GATE-ERROR-1** | `FireflyError::Unauthorized` (401 ONLY; no 302 handling needed) + `PersonalAccessTokenMissing` (empty; unchanged) + `Http` (reqwest; network) + `UnexpectedStatus` (other) |
| **GATE-MESSAGE-1** | `"firefly_personal_access_token invalid or expired — regenerate in Firefly profile → API tokens → update FIREFLY_PERSONAL_ACCESS_TOKEN (see docs/engineering/runbook.md § Omniflow external deploy)"` |
| **GATE-302-HANDLING** | No handling needed — content negotiation ensures 401 for app requests |
| **GATE-PREFLIGHT-1** | Defer (as discovery decided) |
| **GATE-TEST-1** | wiremock integration test: mock 401 → assert `FireflyError::Unauthorized` |
| **GATE-DEC-1** | No new DEC (error taxonomy is implementation detail; document in architecture § BUG-0027) |

## Code map (execute targets, read-only confirmed)

| File:Line | What | Change |
|-----------|------|--------|
| `backend/src/firefly/mod.rs` L20-37 | `FireflyError` enum | Add `Unauthorized` variant |
| `backend/src/firefly/mod.rs` L128-158 | `request()` retry loop | Match `StatusCode::UNAUTHORIZED` → `Err(FireflyError::Unauthorized)` |
| `backend/src/sync/mod.rs` L249-256 | `execute_run` error persistence | No change — `e.to_string()` captures new variant Display |
| `backend/src/sync/mod.rs` L561-564 | `exchanges_only_terminal` | No change — `e.to_string()` pass-through |
| `backend/src/config/mod.rs` L89-90 | `pat_configured()` | No change — emptiness check preserved (separate `PersonalAccessTokenMissing` path) |
| `backend/tests/firefly_integration.rs` | existing wiremock | Add `unauthorized_maps_to_unauthorized_error` test |

## Test strategy

- Framework: wiremock 0.6 (already in `backend/Cargo.toml`)
- Pattern: match `tests/firefly_integration.rs` — `MockServer::start()`, mount 401 response, assert `FireflyError::Unauthorized`
- Preserve existing `PersonalAccessTokenMissing` test in `sync/mod.rs` L587-597

## /quick sizing (5 tasks)

| ID | File(s) | Description |
|----|---------|-------------|
| E1 | `firefly/mod.rs` L20-37 | `FireflyError::Unauthorized` variant |
| E2 | `firefly/mod.rs` L128-158 | Match 401 → `Unauthorized` |
| T1 | `tests/firefly_integration.rs` | wiremock 401 test |
| G1 | all files | Regression gates: cargo lib + firefly integration sync tests |
| V1 | operator | PAT regen + redeploy + ≥3 scheduled syncs verify |

Under `SPRINT_MAX_TASKS=12`. **Backend-only** (no frontend changes; `error_message` pass-through). Recommend `/quick`.

## Recommended next phase

`/architecture` (role: tech-lead) — formalize error taxonomy contract, freeze `Unauthorized` Display message template, then `/quick` execute.

---

# discovery-20260622-bug0027 — BUG-0027 Firefly sync 401 Unauthorized (PAT invalid/expired)

**From:** PO **To:** Tech Lead **Bug:** BUG-0027 **Run:** `auto-20260622-bug0027`
**Date:** 2026-06-22 **Next phase:** `/research` (role: tech-lead)
**Intake evidence:** `handoffs/intake_evidence/intake-20260622-firefly-sync-401.json` (read-only)

## Discovery summary

Sync status confirms persistent **401 Unauthorized** since deploy 2026-06-16. PAT is present (980 chars) but Firefly rejects Bearer auth with **302→/login** → HTML 200 (curl probe). Settings endpoint confirms config loads correctly (`firefly_base_url=http://firefly:8080`, `firefly_auth_method=personal_access_token`). Root cause: PAT invalidated/revoked in Firefly (container recreation, secret rotation, or manual profile revocation).

## Sub-defects confirmed

| AC | Verdict | Fix type | Key evidence |
|----|---------|----------|--------------|
| **CB** — sync succeeds after PAT regen | **OPS** | `GET /api/v1/sync/status` shows `state: failed`, `error_message: unexpected status 401 Unauthorized`; PAT present (980 chars) but Firefly returns 302→/login on API calls |
| **CC** — clear user-facing diagnosis (PAT invalid/missing vs unreachable) | **CODE (Option A)** | `firefly/mod.rs` L158 `FireflyError::UnexpectedStatus` generic — no auth classification; `sync/mod.rs` L250-254 persists raw `e.to_string()` |
| **CD** — ≥3 scheduled syncs succeed post-regen | **OPS** | Verify after CB regression — monitor 3 scheduled runs |

## Hypotheses verdicts

| ID | Verdict | Evidence |
|----|---------|----------|
| **H1: PAT expiry/invalidation (Firefly)** | **CONFIRMED (primary)** | PAT present (980 chars) but Firefly returns 302→/login; R-0057 confirms Firefly API uses Bearer PAT; container recreation revokes tokens |
| **H2: no structured error classification** | **CONFIRMED** | `FireflyError::UnexpectedStatus(status)` + `e.to_string()` → "unexpected status 401 Unauthorized"; existing `PersonalAccessTokenMissing` only catches **empty** PAT (config L89 emptiness check) |
| **H3: startup preflight (PAT validity probe)** | **PARTIAL / DEFER** | Optional nice-to-have (future US); not required for BUG-0027 CB/CC/CD |

## Root-cause chain (code + live)

| Layer | Finding |
|-------|---------|
| **Config** | `FireflyConfig::pat_configured()` (L89) checks `!token.trim().is_empty()` — emptiness only |
| **Sync preflight** | `execute_run()` L209 `pat_configured()` check → `PersonalAccessTokenMissing` when empty; **does not detect invalid PAT** |
| **HTTP request** | `firefly::request()` L148-158: success → `json()`; 5xx/429 → retry; else → `UnexpectedStatus(status)` (generic) |
| **Error persistence** | `sync/mod.rs` L250-254: `finish_sync_run(..., e.to_string())` → "unexpected status 401 Unauthorized" |
| **Live probe** | PAT 980 chars + 302→/login → Firefly rejects token (invalid/expired in Firefly profile) |
| **Curl probe** | `Bearer <PAT>` → 302 → `/login` → 200 HTML; app may receive 200 HTML (unexpected) or Firefly returns 401 directly for API calls (per OpenAPI spec) |

## Architectural options (discovery frozen)

| Option | Approach | Scope | PO recommendation |
|--------|----------|-------|-------------------|
| **A** | Error classification: `FireflyError::Unauthorized` (401/302) → "PAT invalid or expired — regenerate in Firefly profile → API tokens → update FIREFLY_PERSONAL_ACCESS_TOKEN" | Small blast radius | **Preferred** — satisfies CC |
| **B** | Option A + `/health` startup preflight (PAT validity probe, cache state) | Medium | Defer to future US (not required for BUG-0027) |
| **C** | Option A + B + `/api/v1/sync/test-pat` endpoint | Medium-high | Overkill; defer |

**Decision gate: (c) Both** — CB/CD are ops (PAT regen), CC requires code (Option A error classification). Not pure-ops.

## Frozen gates (discovery → research carry)

| Gate | Question | PO default |
|------|----------|------------|
| **GATE-ERROR-1** | Error classification taxonomy | `Unauthorized` (401/302 → "PAT invalid/expired") / `PersonalAccessTokenMissing` (empty → "PAT missing") / `Http` (reqwest → "Firefly unreachable") |
| **GATE-MESSAGE-1** | User-facing message for `Unauthorized` | "Firefly PAT invalid or expired — regenerate in Firefly profile → API tokens → update FIREFLY_PERSONAL_ACCESS_TOKEN (see docs/engineering/runbook.md § Omniflow external deploy)" |
| **GATE-PREFLIGHT-1** | Startup health probe for PAT validity? | **Defer** — not required for CB/CC/CD; future US |
| **GATE-TEST-1** | Regression test | Integration test: mock Firefly returns 401 → assert `Unauthorized` variant + sync run `error_message` contains "PAT invalid"; existing `PersonalAccessTokenMissing` test preserved |
| **GATE-DEC-1** | New DEC? | **No** — error classification is implementation detail; document in architecture only |

## Research questions (carry from R-0099)

1. Confirm reqwest redirect behavior: does `reqwest::Client` follow 302→/login by default? If so, error may be "200 OK" (HTML) → `json()` parse fail → `reqwest::Error` (not `UnexpectedStatus`). Live probe shows "401 Unauthorized" — Firefly may return 401 directly for API calls with invalid Bearer (per OpenAPI spec, not 302).
2. Verify Firefly API auth contract: invalid/expired PAT → 401 Unauthorized (per [Firefly III OpenAPI](https://api-docs.firefly-iii.org/#authentication))?
3. Freeze Option A error classification: `Unauthorized` variant + user-facing message + sync run `error_message` persistence?
4. Integration test design: mock Firefly returns 401 → assert `Unauthorized` variant; preserve existing `PersonalAccessTokenMissing` (empty PAT) test?
5. Blast radius: `firefly/mod.rs` (error enum + request method), `sync/mod.rs` (error persistence), no frontend change (pass-through `error_message`)?

## `/quick` sizing

| Estimate | Tasks |
|----------|-------|
| **5–6 atomic tasks** | `FireflyError::Unauthorized` variant; `request()` 401/302 mapping; `execute_run` error message persistence; integration test; V1 operator smoke (CB/CD after PAT regen) |

Under `SPRINT_MAX_TASKS=12`. **Recommend `/quick`** (same track as BUG-0022/0024/0025/0026 bug fixes).

## Acceptance rows (canonical: `docs/product/acceptance.md` BUG-0027)

- **(CB)** After regenerating Firefly PAT in Firefly profile → API tokens and updating `FIREFLY_PERSONAL_ACCESS_TOKEN` in `/workdir/financegoblin/.env` (or operator `.env`) + recreating container, `GET /api/v1/sync/status` shows `state: completed` after next scheduled run; manual "Sync now" returns `status: completed` with non-zero entity counts; no `error_message` containing `401` or `Unauthorized`. Live Firefly mirror reflects current data. **(OPS)**
- **(CC)** App surfaces clear user-facing diagnosis on `/sync` when PAT auth fails (distinguishing "PAT expired/invalid" from "PAT missing" and "Firefly unreachable") — operator knows to regenerate. **(CODE — Option A)**
- **(CD)** Regression: subsequent scheduled syncs succeed across ≥3 cycles (no silent 401 recurrence) after PAT regen. OIDC-enabled and omniflow external deploy regression checks pass. **(OPS)**

## Decision gates (discovery frozen)

- **GATE-ERROR-1**: `Unauthorized` (401/302) / `PersonalAccessTokenMissing` (empty) / `Http` (network/DNS)
- **GATE-MESSAGE-1**: "Firefly PAT invalid or expired — regenerate in Firefly profile → API tokens → update FIREFLY_PERSONAL_ACCESS_TOKEN"
- **GATE-PREFLIGHT-1**: Defer startup health probe to future US
- **GATE-TEST-1**: Integration test mock 401 → `Unauthorized` variant
- **GATE-DEC-1**: No new DEC (implementation detail; document in architecture)

## Related work

**BUG-0002** DONE (**Q0008** prior PAT-empty case on rebuild); **BUG-0025** DONE (**Q0034**); [R-0057](docs/engineering/research.md#r-0057--firefly-iii-api-docs-discovery-post-bug-0001) (Firefly PAT /auth contract); [R-0099](docs/engineering/research.md#r-0099--bug-0027-firefly-sync-401-unauthorized-discovery-pat-invalidexpired) (this discovery)

## Recommended next phase

`/research` (role: tech-lead) — confirm reqwest redirect behavior (302 → 401 mapping); verify Firefly API 401 vs 302 contract; freeze Option A error classification; design integration test; size `/quick` tasks.

---

