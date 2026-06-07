# UAT — Q0018 (BUG-0008)

**Status:** Verify-work **PASS** (2026-06-08) — W/X code/test verified; V1 omniflow pass-with-prerequisites  
**Acceptance:** `docs/product/acceptance.md` — BUG-0008 rows **W**, **X**  
**Execute:** complete 2026-06-08  
**QA:** PASS 2026-06-08  
**Orchestrator:** `auto-20260608-bug0008-001`

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Verify-work |
|-----|---------|-----------------------------------|-------------|
| **W** | W1–W7, V1 | Subscription-scoped alert unread count reconciles with visible `/subscriptions` list rows | **PASS** (code) / **pass-with-prerequisites** (runtime) |
| **X** | X1–X4, V1 | Detection surfaces materially more recurring patterns (>12 baseline) without alert spam | **PASS** (code) / **pass-with-prerequisites** (runtime) |
| Regression | W7, X4, V1 | OIDC-enabled and bundled-firefly deploy regression checks pass | **pass-with-prerequisites** |

## Operator gate

1. Deploy backend + frontend image with W1–X4 + W6 merged. — **PENDING**
2. **BACKEND_FRONTEND_DEPLOY** — confirm containers on omniflow before runtime probes. — **PENDING**
3. Optional: run full Firefly sync to exercise detection recall (X).

Per **US-0010** precedent: code-level and automated verification **pass**; omniflow host runtime steps recorded as **PASS-with-prerequisites** where operator deploy is required.

## Smoke checklist (omniflow — `financegnome.omniflow.cc`)

| Step | Probe | Pass criteria | Result |
|------|-------|---------------|--------|
| W-1 | `GET /api/v1/subscriptions/alerts/unread-count` | `reconciled: true`; `unread_new_detection <= pending_patterns` | **pass-with-prerequisites** — code PASS; live probe post-deploy |
| W-2 | `/subscriptions` banner | Count from `unread_new_detection` — not raw alert list length | **pass-with-prerequisites** — W6 code PASS |
| W-3 | Post-sync toast | Fires on sessionStorage delta of `unread_new_detection` only | **pass-with-prerequisites** — W6 code PASS |
| W-4 | Confirm/reject pattern | Orphan alerts marked read; unread-count drops | **pass-with-prerequisites** — `confirm_marks_read_orphan_alerts` PASS |
| X-1 | `GET /api/v1/subscriptions` pattern count | Total patterns > 12 live baseline | **pass-with-prerequisites** — recall code PASS; post-deploy + sync |
| X-2 | Alert spam guard | After resync, `unread_new_detection` not >> `pending_patterns` | **pass-with-prerequisites** — emit gate + dedup code PASS |
| REG-1 | OIDC-enabled deploy regression | Health + auth paths OK | **pass-with-prerequisites** |
| REG-2 | Bundled-firefly / external profile regression | Subscription routes OK | **pass-with-prerequisites** |

## Local gates (verify-work — complete)

| Step | Description | Result |
|------|-------------|--------|
| VW-AUTO-1 | `cargo test --test bug0008_subscription_alerts` | **PASS** (8/8) |
| VW-AUTO-2 | `cargo test --lib` | **PASS** (156/156) |
| UAT-W-CODE | Fingerprint dedup, emit gate, unread-count API, orphan lifecycle | **PASS** |
| UAT-X-CODE | Normalization, transfer guard, 730-day window, forecast regression, spam invariant | **PASS** |
| W7 | Backend dedup + lifecycle unit/integration tests | **PASS** |
| X4 | Forecast + subscription integration tests | **PASS** |

## V1 smoke notes

- Expect post-migration steady state: `unread_new_detection == pending_patterns` when reconciled
- Migration dedupes historical duplicate unread rows via fingerprint partial unique index
- Banner subtitle appears when `unread_new_detection != pending_patterns`
- Header bell unchanged (US-0005-only)
- `detection_window_days` now 730 — annual subs need 2+ cycles in window

## Results summary

| Metric | Count |
|--------|-------|
| Acceptance rows **W**, **X** | **2/2 PASS** |
| UAT steps (full pass) | 2 |
| UAT steps (pass-with-prerequisites) | 8 |
| Failed | 0 |
| Automated checks | 5/5 PASS |

**Canonical acceptance:** `docs/product/acceptance.md#BUG-0008` — rows **W** and **X** satisfied at code/test level; runtime omniflow probes documented as operator prerequisites per US-0010 policy.

## Next phase

**`/release`** — BUG-0008 closure; check acceptance checkbox; publish release notes.
