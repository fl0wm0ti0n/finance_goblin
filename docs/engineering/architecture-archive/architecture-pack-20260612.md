# Architecture archive pack (2026-06-12)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 4
- Retained units in hot file: 9
- First archived heading: `# BUG-0019 — Grafana metrics wrong (cashflow zeros, sync entity counts)`
- Last archived heading: `## US-0016 — Root README for operators and contributors (living documentation)`
- Verification tuple (mandatory):
  - archived_body_lines=905
  - preamble_lines=10
  - retained_body_lines=2309

---

# BUG-0019 — Grafana metrics wrong (cashflow zeros, sync entity counts)

**Status:** architecture complete (2026-06-10)  
**Discovery:** `discovery-20260610-bug0019` in `handoffs/po_to_tl.md`  
**Research:** [R-0089](research.md#r-0089--bug-0019-grafana-cashflow-zeros-account_id-default--sync-entity-counts-per-run-cursor)  
**Decisions:** **DEC-0108** (provisioning-only fix); extends **DEC-0107** (separate read path — unchanged)  
**Acceptance:** `docs/product/acceptance.md` rows **BG** (CA), **BH** (CB)

### Root cause (frozen, R-0089)

- **CA:** `$account_id` variable `"sort": 1` re-sorts options alphabetically after the SQL `ABS(balance) DESC` order → default becomes unfunded Cash wallet (116, all-zero rows) instead of funded 114 (731 non-zero rows); no `current` set, kiosk embed passes no `var-account_id`. Query/selection problem — data for 114 is non-zero. Same latent defect in `forecast-horizons.json`.
- **CB:** `platform-health.json` panel 2 reads `sync_cursors.records_synced` verbatim; `upsert_cursor` overwrites it with the **per-run** window count → 0-new-tx incremental run writes `transactions: 0` vs 922 mirror rows. Panel semantics wrong, cursor semantics correct.
- **Not root cause:** embed transport (BUG-0001B fixed), retention race (rows inserted before `mark_success`; retention deletes only old successes), data-side zeros for 114.

### Architecture contract (DEC-0108) — provisioning-only

| Change | File | Detail |
|--------|------|--------|
| CA-1 | `grafana/provisioning/dashboards/analytics/cashflow.json` | `$account_id` variable `"sort": 0` + add `current` (first option); bump `version` |
| CA-2 | same file, panels 1–3 | latest-success subquery gains `AND model_kind = 'baseline'` (align with API default) |
| CA-3 | `grafana/provisioning/dashboards/analytics/forecast-horizons.json` | `$account_id` `"sort": 0` + `current`; bump `version` (panels already filter `$forecast_variant`) |
| CB-1 | `grafana/provisioning/dashboards/platform-health.json` panel 2 | replace `rawSql` with per-entity mirror `COUNT(*)` UNION ALL (transactions, accounts, categories, budgets, tags, piggy_banks) LEFT JOIN `sync_cursors` for `last_successful_sync_at`; bump `version`; full SQL sketch in DEC-0108 |

**Forbidden:** backend/frontend/migration edits (`upsert_cursor`, `sync_transactions`, `AnalyticsEmbedPage.tsx`); hardcoding account id 114.

**Rejected alternatives:** embed-forwarded `var-account_id` (coupling, deferrable); household SUM panels (changes BG meaning); cumulative `records_synced` (overlap re-fetch overcounts); backend `total_records` column (duplicates live SQL). Full table: DEC-0108.

### Deploy / operator steps

1. Apply the three JSON edits; bump each dashboard `version`.
2. **Re-provision Grafana:** `docker compose restart grafana` (provisioning reload required — JSON dashboards are loaded at startup/scan interval).
3. Verify via kiosk embed (`AnalyticsEmbedPage`) **and** direct Grafana URL (covers no-`var-account_id` path).
4. Rollback: `git revert` of the JSON files + Grafana restart (no schema/data state).

### Verification gates

| Gate | Proof |
|------|-------|
| **BG** | Cashflow panels 1–2 non-zero (negative) for default account = 114; matches `GET /api/v1/forecast/monthly?account_id=114` (25 points, non-zero from Jul 2026) |
| **BH** | Platform Health `transactions` = `SELECT COUNT(*) FROM transactions` (922) after Full sync **and** after a subsequent 0-new-tx incremental run |
| Static | JSON checks: `account_id` `sort: 0` in both dashboards; platform-health panel 2 SQL references mirror tables |
| OIDC | Re-run BG/BH on omniflow profile (provisioning-only blast radius) |

### Risks

| Risk | Mitigation |
|------|------------|
| Cached dashboard after provisioning edit | Restart Grafana + `version` bump |
| Fresh install (all balances 0) → arbitrary default account | Acceptable; runbook note |
| `model_kind='baseline'` freezes cashflow panels off ML | Accepted; future `$forecast_variant` wiring per R-0051 |
| Mirror COUNT(*) ≠ "synced" if Firefly-side deletions linger | BH specifies mirror truth; reconciliation out of scope |

**Out of scope (flag to PO):** 43 `ml_enhanced` computations stuck `status='running'` accumulate unbounded (retention prunes successes only) — recommend new backlog bug.

`isolation_scope`: artifact + repo source reads only; no host `.env` / secrets read.

---

# BUG-0018 — Alert evaluation SQL failure (balance ambiguous)

**Status:** architecture complete (2026-06-10)  
**Full section:** `docs/engineering/architecture-archive/architecture-pack-20260609-a.md` § BUG-0018  
**Decisions:** **DEC-0107** · **Sprint:** `/quick` **Q0026** (PLANNED — BE1 + T1 + V1) · **Acceptance:** BE, BF

---

# BUG-0016 — SPA deep links return HTTP 404

**Status:** architecture complete (2026-06-09)  
**Discovery:** `discovery-20260609-bug0016` in `handoffs/archive/po-to-tl-pack-20260609-a.md`  
**Research:** [R-0086](research.md#r-0086--bug-0016-spa-deep-link-fallback-axum-vs-traefik)  
**Decisions:** **DEC-0104** (Axum SPA `index.html` fallback); extends **DEC-0057** (Grafana proxy ordering — unchanged); **no Traefik label change**  
**Sprint:** `/quick` **Q0024** (PLANNED — AX1 + AX2 + V1)  
**Acceptance:** `docs/product/acceptance.md` row **AX** (execute: **AX1**, **AX2**, **V1**)  
**Spec-pack:** `docs/engineering/spec-pack/BUG-0016-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**Related:** UI-001 (UI audit curl evidence); BUG-0009 (analytics 404 advisory superseded for shell routing only)

### Root cause (frozen)

`build_router` merges `health`, `analytics::grafana_routes`, and `api::routes` before a `tower_http::ServeDir` fallback — ordering is correct per R-0056 / **DEC-0057**. The defect is the fallback itself: plain `ServeDir` returns **404** when no file exists at the request path (e.g. `/forecast`). `/` works because `index.html` exists at the directory root; client-side sidebar navigation works because the shell is already loaded.

**Not root cause:** React Router misconfiguration, page components, Traefik host rules, or OIDC redirect handling.

`isolation_scope`: artifact + repo source reads; no host `.env` / `.env_prod` secrets read.

### Architecture contract (DEC-0104)

```text
BUG-0016
├── AX1 — SPA fallback in build_router (P0)
│   └── ServeDir::fallback(ServeFile::new(index.html)) → HTTP 200
├── AX2 — Integration tests (P0)
│   └── Deep links 200; /api/v1/* and /analytics/grafana/* ≠ HTML
└── V1 — verify-work curl + browser smoke (P0)
    └── AX matrix on :18080; omniflow hard-refresh; OIDC /callback
```

**Deploy:** Single backend change; rebuild `flow-finance-ai` image. No Traefik label edits (R-0086 §5).

### Route ordering (must-not-break)

| Order | Prefix | Handler | SPA fallback? |
|-------|--------|---------|---------------|
| 1 | `/health` | `health::routes` | No |
| 2 | `/analytics/grafana/*` | `analytics::grafana_routes` (**DEC-0057**) | No — proxy JSON/assets/WebSocket |
| 3 | `/api/v1/*` | `api::routes` (JWT when OIDC on) | No — JSON 404/401 preserved |
| 4 | * | `ServeDir` + `index.html` fallback | Yes — client routes + `/callback` |

**`/callback`:** No Axum redirect or catch-all rewrite to `/`. Serving `index.html` at `/callback` is correct — `App.tsx` registers `/callback` outside `ProtectedRoute` for OIDC token exchange.

### Implementation contract

```rust
use tower_http::services::{ServeDir, ServeFile};

let index = static_dir.join("index.html");
let spa = ServeDir::new(static_dir).fallback(ServeFile::new(index));
router = router.fallback_service(spa);
```

| API | Status on deep link | Use? |
|-----|---------------------|------|
| `ServeDir::fallback(ServeFile::new(index.html))` | **200** | **Yes** — matches AX |
| `ServeDir::not_found_service(ServeFile::new(index.html))` | **404** (body = index.html) | **No** |
| Plain `ServeDir` (current) | **404** empty | Current bug |

Apply identically to the `frontend/dist` dev branch.

### Regression matrix (acceptance AX)

**Primary AX paths (curl — localhost `:18080`, `AUTH_DEV_BYPASS`):**

| Path | Expected after fix |
|------|-------------------|
| `GET /forecast` | 200, `text/html`, body contains `<div id="root">` or Vite shell marker |
| `GET /subscriptions` | 200 HTML shell |
| `GET /planning` | 200 HTML shell |
| `GET /sync` | 200 HTML shell |
| `GET /analytics/cashflow` | 200 HTML shell |

**Expanded same-contract paths:** `/wealth`, `/alerts`, `/chat`, `/settings`, `/analytics/{platform-health,budgets,portfolio,subscriptions,forecast-horizons}`.

**Protected prefixes (must stay non-HTML):**

| Path | Expected |
|------|----------|
| `GET /api/v1/health` or representative API route | JSON, not HTML |
| `GET /analytics/grafana/api/health` | Proxy response, not SPA index |
| `GET /assets/{hashed}.js` | 200 static file with correct `Content-Type` |
| `GET /api/v1/nonexistent` | JSON 404, not `index.html` |

**Browser smoke (operator):** Hard-refresh Forecast, Planning, Analytics embed; bookmark reopen; omniflow with Traefik `auth` + optional OIDC; `/callback?code=…&state=…` completes.

### Alternatives rejected

| Alternative | Why rejected |
|-------------|--------------|
| Traefik catch-all / nginx sidecar | Duplicate responsibility; no acceptance gain (R-0086 §2) |
| `not_found_service` | 404 status fails AX curl gate |
| Redirect unknown paths to `/` | Breaks bookmarked URLs |
| Backend `/callback` handler | Conflicts with React OIDC flow |
| `axum_extra::SpaRouter` | Unnecessary nesting; Vite already emits `/assets/*` at root |

### Risks

| Risk | Mitigation |
|------|------------|
| API paths receive `index.html` | Preserve merge order; AX2 integration test |
| Grafana proxy swallowed by SPA | `grafana_routes` before fallback; test `/analytics/grafana/…` |
| `index.html` missing in image | Dockerfile copies `dist` → `/app/static` |
| Traefik `/analytics/*` no-auth exposes SPA slug | Pre-existing US-0011 tradeoff; document, do not widen |
| OIDC `/callback` broken by redirect | Do not add `Redirect` fallback |

### Decisions

| ID | Topic | Summary |
|----|-------|---------|
| **DEC-0104** | SPA fallback | Axum-only; `ServeDir::fallback(ServeFile)` → HTTP 200; Traefik pass-through |

Full record: `decisions/DEC-0104.md`

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **AX** | AX1, AX2, V1 | Curl matrix + browser smoke + OIDC `/callback` |

### Next phase

`/plan-verify` — audit Q0024 acceptance AX coverage; then `/execute`.

---

## US-0016 — Root README for operators and contributors (living documentation)

**Status:** Architecture complete (2026-06-08)  
**Discovery:** `discovery-20260608-us0016` in `handoffs/po_to_tl.md`  
**Research:** [R-0066](research.md#r-0066--root-readme-split-layout-and-living-doc-maintenance), [R-0067](research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks)  
**Decisions:** **DEC-0070** (template parity posture, Product status placement, maintenance hooks); extends doc-profile split layout (US-0077 / runbook § documentation profile validation)  
**Sprint:** Single sprint recommended (~6–8 tasks) under `SPRINT_MAX_TASKS` (12)  
**Acceptance:** `docs/product/acceptance.md` § US-0016 (6 rows)  
**Spec-pack:** `docs/engineering/spec-pack/US-0016-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**User-guide:** No per-story guide required; root README links `docs/user-guides/` when `USER_GUIDE_MODE=1` (`docs/user-guides/US-xxxx.md` schema per US-0032)

### Problem

Root `README.md` is **missing**. First clone fails `validate_doc_profile.py` with `README.md missing`. Operators and contributors lack a single entry document for product purpose, compose Quickstart, and doc navigation. The living-doc promise requires curated status updates at phase boundaries without per-commit automation or backlog duplication.

`isolation_scope`: artifact + repo source only; no host `.env` / secrets read.

### Architecture contract (DEC-0070)

```text
US-0016
├── R1 — Root README split layout (P0)
│   └── README.md: 5 user H2s + ## Contributing; Flow Finance AI content
├── R2 — Product status subsection (P0)
│   └── ### Product status under ## Purpose; 8 bullets max; backlog link
├── R3 — Related documentation + compose (P0)
│   └── user-guides, runbook, spec paths; minimal/bundled-firefly/external commands
├── R4 — Validator + CI gate (P0)
│   └── validate_doc_profile --no-template-parity until template/ ships
├── R5 — Runbook maintenance hooks (P0)
│   └── § README maintenance (US-0016); release + refresh-context checklist
├── R6 — Developer shard pointer (P1)
│   └── docs/developer/README.md workflow note
└── T1 — Template flip gate (deferred)
    └── Drop --no-template-parity when full template/ tree lands (out of US-0016 default execute)
```

**Out of scope:** Full `template/` installer mirror; auto-README on every commit; its-magic framework manual; application code changes.

### R1 — Split layout (frozen)

Active profile: `DOC_AUDIENCE_PROFILE=both`, `DOC_DETAIL_LEVEL=balanced` (merged scratchpad).

| Surface | Required elements |
|---------|-------------------|
| Root `README.md` | H2: `Purpose`, `Quickstart`, `Examples`, `Limitations`, `Related documentation` (exact titles per `doc_profile_lib.USER_KEY_TO_H2`) |
| Root pointer | `## Contributing` → [`docs/developer/README.md`](docs/developer/README.md) |
| Forbidden in root | Any `DEV_*` H2 titles (`doc_profile_lib.dev_h2_forbidden_in_root`) |
| Developer shard | `DEV_PREREQS`, `DEV_WORKFLOW`, `DEV_QUALITY_GATES`, `DEV_ARCHITECTURE` in `docs/developer/README.md` only |

**H2 budget:** `count_profile_root_h2s` counts required `USER_*` titles only — `## Contributing` and extra H2s do not consume budget ([R-0067](research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks) §2). For `(both, balanced)`: 5 required user H2s vs budget 8.

**Content sources ([R-0066](research.md#r-0066--root-readme-split-layout-and-living-doc-maintenance)):**

| Section | Source |
|---------|--------|
| Purpose | Product value proposition; link backlog for history |
| Quickstart | Compose profiles from `.env.example` (minimal, bundled-firefly, external omniflow) |
| Examples | Sync + analytics routes; copy-paste friendly |
| Limitations | Known sharp edges; unsupported envs |
| Related documentation | `docs/user-guides/`, `docs/engineering/runbook.md`, architecture/decisions index, spec-pack paths when `SPEC_PACK_MODE=1` |

**Alternatives rejected:** DEV_* sections in root; dedicated `## Product status` H2; nesting status under Related documentation ([R-0067](research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks)).

### R2 — Product status (frozen)

| Contract | Value |
|----------|-------|
| Placement | `### Product status` immediately under `## Purpose` |
| Format | `{US-xxxx\|BUG-xxxx} — {one-line outcome}` |
| Order | Reverse-chronological (newest first) |
| Cap | **8** bullets — drop oldest |
| History | Link `docs/product/backlog.md`; never duplicate acceptance tables |

**Anti-patterns:** Full backlog dump; secrets; placeholder stubs left after release.

### R3 — Template parity posture (frozen)

| Repo state | Command | AC-6 |
|------------|---------|------|
| `template/` **absent** (current) | `python scripts/validate_doc_profile.py --repo . --no-template-parity` | Satisfied vacuously ("when tree exists") |
| `template/` **present** | Default (no flag) | Requires `template/README.md` + `template/docs/developer/README.md` parity |

**Rejected:** Partial stub `template/README.md` only — parity requires dev shard ([R-0067](research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks) §1).

**Flip gate:** Remove `--no-template-parity` in the **same change set** that adds the full `template/` mirror. Document in runbook § README maintenance.

### R4 — Maintenance hooks (frozen)

Phase-boundary updates only — not per-commit ([R-0066](research.md#r-0066--root-readme-split-layout-and-living-doc-maintenance), [R-0067](research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks) §3).

#### Release (`/release`)

After backlog reconciliation (≈ step 10), before runbook readiness (≈ step 14):

1. For each **US** or **BUG** in target sprint → **DONE** / **CLOSED**, append one Product status bullet.
2. Trim to 8 most recent entries.
3. Run `python scripts/validate_doc_profile.py --repo . --no-template-parity` — non-zero → fail closed; remediation → runbook § README maintenance.

#### Refresh-context (`/refresh-context`)

After backlog status reconciliation:

1. If closures since prior refresh, verify Product status includes closed id(s); update if missing.
2. If README or doc-profile surfaces touched, run validator with `--no-template-parity`.

#### Developer shard

One sentence in `docs/developer/README.md` § Workflow or Quality gates pointing to runbook § README maintenance.

#### Runbook (execute)

New subsection **`README maintenance (US-0016)`** under § documentation profile validation — embed hooks above; document both validator commands and template flip gate.

### File touch list (frozen)

| Path | Task | Change |
|------|------|--------|
| `README.md` | R1–R2 | Create; split layout + Product status + content |
| `docs/developer/README.md` | R6 | Workflow pointer to README maintenance |
| `docs/engineering/runbook.md` | R5 | § README maintenance (US-0016) |
| `tests/run-tests.sh` or CI doc gate | R4 | `validate_doc_profile --no-template-parity` |
| `.env.example` | R1 | Reference only for Quickstart content (no structural change required) |

**No touch:** Application source, compose behavior, `template/` tree (deferred).

### Validation strategy

| Check | Type | Pass criteria |
|-------|------|---------------|
| AC-1 Split layout | `validate_doc_profile.py` | All required user H2s present with non-stub content |
| AC-2 Contributing | Validator + manual | `## Contributing` present; zero DEV_* H2 in root |
| AC-3 Related docs | Manual + optional-mode warnings | user-guides, runbook, compose commands; spec crosslink when `SPEC_PACK_MODE=1` |
| AC-4 Validator | CI + local | Exit 0 with `--no-template-parity` |
| AC-5 Runbook | Doc review | § README maintenance with release + refresh hooks |
| AC-6 Template | Vacuous | N/A until `template/` exists |

### Risks

| Risk | Mitigation |
|------|------------|
| Stale Product status | Release fail-closed validator + refresh-context verify ([R-0067](research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks) §3) |
| `--no-template-parity` left on after template ships | DEC-0070 flip gate + runbook note |
| Scope creep (backlog dump) | 8-bullet cap + backlog link ([R-0066](research.md#r-0066--root-readme-split-layout-and-living-doc-maintenance)) |
| Operator confusion (two validator commands | Runbook documents both; architecture cites current posture |

### Decisions (US-0016)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0070 | Root README living documentation | `--no-template-parity` until full `template/`; `### Product status` under Purpose; release + refresh-context hooks |

Full record: `decisions/DEC-0070.md`

### Acceptance mapping

| AC | Architecture slice | Verify |
|----|-------------------|--------|
| AC-1 | R1 | Validator + content review |
| AC-2 | R1 | No DEV_* in root; Contributing pointer |
| AC-3 | R1, R3 | Related docs + compose commands |
| AC-4 | R4 | `validate_doc_profile --no-template-parity` exit 0 |
| AC-5 | R5 | Runbook § README maintenance |
| AC-6 | T1 (deferred) | Vacuous until `template/` lands |

### Next phase

`/sprint-plan` — decompose 6 acceptance criteria; expect ~6–8 tasks (README content, Product status seed, runbook hooks, dev shard pointer, CI validator flag). Single sprint under `SPRINT_MAX_TASKS` (12).

---

## BUG-0008 — Subscription alerts vs list mismatch & under-detection

**Status:** architecture complete (2026-06-08)  
**Discovery:** `discovery-20260608-bug0008` in `handoffs/po_to_tl.md`  
**Research:** [R-0068](research.md#r-0068--bug-0008-subscription-alert-dedup-unread-count-contract-orphan-lifecycle), [R-0069](research.md#r-0069--bug-0008-detection-recall-levers-ai-path-boundary); addenda R-0009–R-0013  
**Decisions:** **DEC-0071** (W bundle); **DEC-0072** (X Phase 1 recall)  
**Sprint:** `/quick` **Q0018** (recommended)  
**Acceptance:** `docs/product/acceptance.md` rows **W**, **X**  
**Spec-pack:** `docs/engineering/spec-pack/BUG-0008-{design-concept,crs,technical-specification}.md`  
**User guide:** `docs/user-guides/BUG-0008.md`  
**Related:** BUG-0004 DONE (J partial — 11 pending baseline); BUG-0007 DONE (coordinate — additive AI JSON only); US-0003 subscription engine; US-0005 unified alerts boundary

### Symptom chain (frozen)

Operator on US-0010 external profile: 922+ transactions synced; subscription alerts unread count diverges from `/subscriptions` list; detection recall below operator expectation.

| Sub | Verdict | Root cause |
|-----|---------|------------|
| **W** | CONFIRMED | Bare `insert_alert` every sync — no fingerprint dedup; banner = raw alert list length (83 unread vs 6 pending live) |
| **X** | CONFIRMED | Payee-only grouping fragments SEPA memos; 365-day window; `category_ids` unused; hardcoded min_emit 60 |

**Live probe (2026-06-08):** 6 pending, 12 total patterns, 83 unread `new_detection` alerts, unified `/api/v1/alerts/unread-count` = 0 (US-0005 — not operator symptom).

`isolation_scope`: artifact + repo source reads; public omniflow API probes (discovery/research); no host `.env` / `.env_prod` secrets read.

### Sequencing (mandatory)

```text
BUG-0008
├── W — DEC-0071 (P0, execute first)
│   ├── W1 — Migration: fingerprint column + partial unique index + backfill dedupe
│   ├── W2 — Repository: insert_alert → upsert_alert (ON CONFLICT)
│   ├── W3 — Detection: emit alert only on new pending or tier increase
│   ├── W4 — API: GET /api/v1/subscriptions/alerts/unread-count
│   ├── W5 — Lifecycle: mark-read orphans on confirm/reject/inactive
│   ├── W6 — Frontend: banner + toast from unread-count API
│   └── W7 — Tests: dedup, reconciled count, lifecycle
└── X — DEC-0072 Phase 1 (P0, after W1–W3 minimum)
    ├── X1 — Payee normalization (SEPA token strip, entity suffix collapse)
    ├── X2 — Transfer-type counterparty priority guard
    ├── X3 — detection_window_days 365 → 730 (config)
    ├── X4 — Integration tests (forecast + subscription regression)
    └── X5 — (Phase 2 gate) Category-aware grouping ≥70% threshold — same sprint if capacity
```

**Rule:** W dedup before X recall threshold tuning. X without W re-amplifies alert spam (discovery risk #1).

**Deploy order:** (W1 → W2 → W3) backend migration + repository → (W4 → W5) API + lifecycle → W6 frontend → (X1 → X2 → X3) recurrence core → X4 tests → optional X5 → operator verify. Single backend PR acceptable if W slices land before X in commit order.

### W — Alert dedup & unread count (DEC-0071)

#### Fingerprint contract (frozen)

| `alert_type` | Fingerprint |
|--------------|-------------|
| `new_detection` | `sub_alert:new_detection:{pattern_id}` |
| `price_change` | `sub_alert:price_change:{pattern_id}:{direction}:{round(new_amount,2)}` |
| `interval_change` | `sub_alert:interval_change:{pattern_id}:{interval_days}` |

Partial unique: `(fingerprint) WHERE read_at IS NULL`. Upsert updates `body`, `sync_run_id`, `created_at` on conflict.

**Files:** `backend/migrations/`, `backend/src/subscriptions/{repository,detection}.rs`.

#### Unread-count API (frozen)

`GET /api/v1/subscriptions/alerts/unread-count` — see **DEC-0071 §2** for response schema.

| UI surface | Field | Reject |
|------------|-------|--------|
| `/subscriptions` banner | `unread_new_detection` | Raw `alerts.length` |
| Post-sync toast | sessionStorage delta on `unread_new_detection` | List poll without dedup |
| Header bell badge | _(unchanged)_ | Combined subscription + unified count |

**Files:** `backend/src/subscriptions/{routes,service}.rs`, `frontend/src/pages/SubscriptionsPage.tsx`.

#### Orphan lifecycle (frozen)

| Event | SQL action |
|-------|------------|
| confirm / reject / inactive | Mark-read unread alerts for `pattern_id` |

**Files:** `backend/src/subscriptions/service.rs` (confirm/reject handlers).

#### BUG-0007 coordinate (frozen)

- **New route only** — no `list_patterns` filter changes
- Additive JSON on existing routes forbidden unless coordinate table updated
- AI tool wrappers unchanged

### X — Detection recall Phase 1 (DEC-0072)

#### Normalization rules (frozen)

| Rule | Example |
|------|---------|
| Strip SEPA reference tokens | `SVWZ+`, card suffixes |
| Collapse legal suffixes | `GmbH`, `AB` |
| Transfer-type guard regex | `SVWZ\|UEBERWEISUNG\|Lastschrift` → prefer `counterparty_name` |

**Files:** `backend/src/recurrence/{normalize,group}.rs`, `backend/src/subscriptions/detection.rs`.

#### Config change (frozen)

`detection_window_days`: **365 → 730** in `backend/config/default.toml`.

#### Phase 2 gate (optional same sprint)

When ≥**70%** txs in payee group share `category_id`, secondary grouping key `cat:{category_id}`. Execute only after Phase 1 probe shows recall gain without W violation.

#### AI boundary (frozen)

| Path | Verdict |
|------|---------|
| In-pipeline LLM | **Reject** |
| Async enrichment job | **Defer** — document in release notes |
| Acceptance **X** footer | Rule improvements in architecture/release notes |

**min_emit_confidence** stays **60** hardcoded until W closed + operator FP review — do not wire to TOML in BUG-0008 execute unless Phase 2 gate opens.

### Task map (Q0018)

| Order | Task | Layer | Est. | Acceptance |
|-------|------|-------|------|------------|
| 1 | **W1** fingerprint migration + backfill | backend migration | 3h | **W** |
| 2 | **W2** upsert_alert repository | backend subscriptions | 2h | **W** |
| 3 | **W3** detection emit gate | backend detection | 2h | **W** |
| 4 | **W4** unread-count API route | backend API | 2h | **W** |
| 5 | **W5** orphan lifecycle hooks | backend service | 1.5h | **W** |
| 6 | **W6** frontend banner + toast | frontend | 2h | **W** |
| 7 | **W7** backend unit/integration tests | backend tests | 3h | **W** regression |
| 8 | **X1** payee normalization | backend recurrence | 3h | **X** |
| 9 | **X2** transfer counterparty priority | backend recurrence | 2h | **X** |
| 10 | **X3** detection window config | backend config | 0.5h | **X** |
| 11 | **X4** forecast + subscription integration tests | backend tests | 2h | **X** regression |
| 12 | **V1** operator verify omniflow | verify-work | 1h | **W**, **X** |

**Count:** 12 tasks (= `SPRINT_MAX_TASKS` 12) → **`/quick` Q0018**; no split. Phase 2 category grouping (**X5**) deferred to follow-up quick if sprint at capacity — recommend execute X5 only if W7+X4 complete under estimate.

**Total estimate:** ~24h (dev ~23h + operator V1 ~1h).

### Test strategy

| Check | Type | Pass criteria |
|-------|------|---------------|
| W1 | Migration | Backfill dedupes duplicates; partial unique index present |
| W2 | Unit | ON CONFLICT upsert; no duplicate unread fingerprints |
| W3 | Unit | No alert on unchanged pending pattern resync |
| W4 | Integration | `reconciled: true` when counts align; JOIN guard |
| W5 | Unit | confirm/reject mark-read orphans |
| W6 | Frontend | Banner uses unread-count; not list length |
| X1–X2 | Unit | SEPA fixture merges under single payee key |
| X3 | Config | 730-day window loads from TOML |
| X4 | Integration | Forecast recurring unaffected or improved |
| Privacy | Regression | OIDC + bundled-firefly deploy smoke |
| V1 | Operator | Banner count ≤ pending; patterns > 12 baseline |

### Decisions (BUG-0008)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0071 | W bundle | Fingerprint dedup + unread-count API + orphan lifecycle + US-0005-only bell |
| DEC-0072 | X Phase 1 | Normalization + counterparty priority + 730-day window; Phase 2 gated; AI deferred |

Full records: `decisions/DEC-0071.md`, `decisions/DEC-0072.md`

### Risks

| Risk | Mitigation |
|------|------------|
| X before W | Frozen task order; W1–W3 before X1 |
| Over-merge (X2) | Transfer-type guard only |
| Forecast regression | X4 integration tests (DEC-0013 shared core) |
| Partial unique + NULL backfill | W1 backfill before NOT NULL |
| BUG-0007 coordinate | Additive unread-count route only |

### Acceptance mapping

| Row | Architecture slice | Verify |
|-----|-------------------|--------|
| **W** | W1–W7 | Reconciled unread-count vs pending; no 33-vs-11 class mismatch |
| **X** | X1–X4 (+ optional X5) | Patterns > 12 baseline; no alert spam (`unread_new_detection <= pending_patterns`) |

Static intake numbers are snapshots — test reconciled semantics and relative recall gain.

### Next phase

**`/sprint-plan` Q0018** — materialize `sprints/quick/Q0018/task.json` from task table; W-before-X task order frozen; then `/plan-verify` → `/execute`.

---

## BUG-0013 — Omniflow analytics regression cluster (budgets MTD, crypto pricing)

**Status:** architecture complete (2026-06-08)  
**Discovery:** `discovery-20260608-bug0013` in `handoffs/po_to_tl.md`  
**Research:** [R-0076](research.md#r-0076--omniflow-analytics-regression-hypotheses-post-us-0015) §5–7, [R-0077](research.md#r-0077--bug-0013-grafana-embed-failed-to-fetch-annotation-runner)  
**Decisions:** **DEC-0079** (AL MTD SQL); **DEC-0080** (AN/AK Bitunix valuation); extends **DEC-0064**, **DEC-0038**, **DEC-0039**; **no DEC-0064 amend** this sprint  
**Sprint:** `/quick` **Q0020** (recommended)  
**Acceptance:** `docs/product/acceptance.md` rows **AI**–**AN** (execute scope: **AL**, **AK**, **AN**; **AI**/**AJ**/**AM** waived or ops-only)  
**Related:** US-0015 DONE (not root cause); BUG-0005 DONE (DEC-0064 ingest — valuation gap residual); BUG-0009/0010 DONE (AI refuted on live probe)

### Symptom chain (frozen)

Operator post-US-0015 cluster on `financegnome.omniflow.cc` decomposes to **two confirmed code defects** and **four non-code items** — not a single US-0015 regression.

| Sub | Verdict | Root cause | Execute |
|-----|---------|------------|---------|
| **AI** | REFUTED (ops/stale) | Baseline forecast non-zero acct **114** after Full sync + recompute | **V1** re-smoke only |
| **AJ** | REFUTED (expected empty) | 0 price-change events in 90d | Optional **AJ1** copy |
| **AK** | CONFIRMED | Linear holdings unpriced → crypto **€0**; performance % needs snapshot history | **AN1** + optional **AK2** |
| **AL** | CONFIRMED | MTD planned sums 730 future plan days (no upper date bound) | **AL1** |
| **AM** | NOT REPRODUCED | curl ds/query + annotations **200** | Waived per **R-0077** |
| **AN** | CONFIRMED | Same as AK — sync OK, EUR valuation missing | **AN1** |

**Live probe (2026-06-08):** acct 114 forecast non-zero; budgets MTD **−150337.6** / actual **0**; Bitunix **7** linear rows all `market_value_eur` NULL; exchange sync success `18:29:40Z`.

`isolation_scope`: artifact + repo source reads; public omniflow curl probes (discovery/research); **no** host `.env` / `.env_prod` secrets read.

### Operator gates (mandatory before V1)

1. **BACKEND_FRONTEND_DEPLOY** — US-0015 image live on omniflow.
2. **Full Firefly sync** — not exchanges-only (wealth snapshot + forecast freshness).
3. **Forecast recompute** — baseline panels on `$account_id=114`.

### Fix slices

```text
BUG-0013
├── AL — DEC-0079 (P0, Grafana-only)
│   └── AL1 — MTD panel id 5: planned CTE `<= CURRENT_DATE`; optional mid-month footnote
├── AN/AK — DEC-0080 (P0, backend)
│   ├── AN1a — bitunix.rs: wallet `data[]` parse + `unrealizedPNL` field keys
│   ├── AN1b — pnl.rs: futures wallet EUR via stablecoin path; linear unrealized USDT→EUR
│   └── AN1c — bitunix.rs tests: array wallet mock + linear unrealized persist
├── Optional UX (P2 — sprint capacity)
│   ├── AJ1 — subscriptions price-changes empty-state copy
│   └── AK2 — portfolio performance % min-snapshot footnote
└── V1 — verify-work omniflow smoke (AL + AN acceptance rows)
```

**Deploy order:** AL1 (Grafana JSON) + AN1 (backend) in one release; operator **Full sync** after deploy; V1 probes.

**Out of scope:** US-0013 ML overlay; MetaMask extension noise; AM1 unless HAR non-200; DEC-0064 exposure_eur display (tier 2 gate).

### AL1 — Budgets MTD upper bound (DEC-0079)

#### Problem

Panel id **5** `planned` CTE:

```sql
... AND pdc.ts >= date_trunc('month', CURRENT_DATE)
```

Missing `<= CURRENT_DATE` → sums entire future plan horizon within dashboard time range.

#### Contract (frozen)

| CTE | SQL addition |
|-----|--------------|
| `planned` | `AND pdc.ts::date <= CURRENT_DATE` |
| `actual` | unchanged |
| Deviation row | `(SELECT total FROM actual) - (SELECT total FROM planned)` with capped planned |

**Files:** `grafana/provisioning/dashboards/analytics/budgets.json` panel **5** `rawSql` only.

**Alternatives rejected:** `$__timeFilter` on summary (includes future); backend MTD view (over-engineered for SQL bug).

**Risks:** UTC `CURRENT_DATE` vs operator TZ — consistent with existing deviation chart UTC usage.

### AN1 — Bitunix futures valuation (DEC-0080)

#### Problem chain

1. Wallet API returns `data: [{...}]` — `parse_futures_wallet` reads `data.account` → **no wallet row**.
2. `recompute_pnl` → `holding_value_eur` → `fx.to_eur(qty, "INJUSDT")` → `Unpriced` → `continue` skips unrealized conversion.
3. Wealth `crypto.subtotal_eur` = sum `market_value_eur` — all NULL → **€0**.

#### Wallet parse contract (AN1a — frozen)

```text
data = body["data"]
account = if data.is_array() → first object with marginCoin/available
          else → data["account"] ?? data
equity = accountEquity | (available + margin + frozen)
asset  = marginCoin | "USDT"
market_value_usd = Some(equity) when asset in {USDT, USDC}
product_type = "futures"
```

Add **`unrealizedPNL`** to position and wallet `parse_f64_field` key lists.

**Test:** Mock array-shaped `data: [{ marginCoin: "USDT", available: "250", ... }]` — assert futures wallet row.

#### Valuation contract (AN1b — frozen)

| `product_type` | Subtotal (`market_value_eur`) | `unrealized_pnl_eur` |
|----------------|------------------------------|----------------------|
| `futures` | `fx.to_eur(quantity, asset)` | from wallet if present |
| `linear` | **skip** — not in `crypto_value_eur` sum | parse payload unrealized; `fx.to_eur(pnl, "USDT")`; **do not** flag `fx_incomplete` for symbol |
| `spot` | existing path | existing path |

**Reject:** Price linear notional into `market_value_eur` (DEC-0064 double-count).

**Files:** `backend/src/exchanges/bitunix.rs`, `backend/src/portfolio/pnl.rs`.

**Deferred tier 2:** `ExchangePriceBook` population in `portfolio/service.rs` (spot tickers).

#### Acceptance mapping (AK/AN)

| Check | Post-AN1 expectation |
|-------|---------------------|
| `GET /api/v1/wealth` `crypto.subtotal_eur` | **> 0** when USDT futures wallet equity &gt; 0 |
| `holdings_top` | Non-empty when wallet priced |
| `unrealized_pnl_eur` on linear rows | Populated from exchange payload |
| Grafana portfolio crypto stat | Non-zero after sync + recompute |
| Performance % | May remain NULL until ≥2 snapshots (**AK2** docs only) |

### AM — Embed Failed to fetch (waived)

Per **R-0077**: curl **200** on ds/query + annotations; console `handleAnnotationQueryRunnerError` likely annotation cancel or WS cosmetic. **No AM1 execute** unless operator HAR shows non-200. Optional: disable built-in dashboard annotation on budgets (P2).

### Task table (sprint-plan input)

| ID | Sub | Task | Files | Priority |
|----|-----|------|-------|----------|
| **AL1** | AL | MTD planned `<= CURRENT_DATE` + optional footnote | `budgets.json` | P0 |
| **AN1** | AN/AK | Wallet parse + pnl linear unrealized EUR + tests | `bitunix.rs`, `pnl.rs` | P0 |
| **AJ1** | AJ | Price-changes empty-state copy | `subscriptions.json` | P2 optional |
| **AK2** | AK | Performance % min-snapshot panel note | `portfolio.json` | P2 optional |
| **V1** | all | verify-work smoke post deploy + Full sync | acceptance AI–AN | P0 |

**Count:** 3 mandatory (AL1, AN1, V1) + 2 optional → **`/quick` Q0020** (≤ `SPRINT_MAX_TASKS` 12).

### Codebase map (BUG-0013 slice)

| Path | Role | Touch |
|------|------|-------|
| `grafana/.../budgets.json` | MTD summary SQL | AL1 |
| `backend/src/exchanges/bitunix.rs` | Wallet/position parse | AN1a,c |
| `backend/src/portfolio/pnl.rs` | EUR valuation loop | AN1b |
| `backend/src/wealth/service.rs` | Crypto subtotal read | verify only |
| `backend/src/fx/service.rs` | USDT→EUR stable path | used by AN1b |

**`/sprint-plan`** — materialize `sprints/quick/Q0020/` from task table; then `/plan-verify` → `/execute`.

---

## BUG-0011 — Planning mode broken (empty plan, compare sums, plan-vs-actual 404)

**Status:** architecture complete (2026-06-08)  
**Discovery:** `discovery-20260608-bug0011` in `handoffs/archive/po-to-tl-pack-20260606-b.md`  
**Research:** [R-0070](research.md#r-0070--bug-0011-planning-mode-compare-delta-empty-state-api-first-run-ux); addenda [R-0015](research.md#r-0015--plan-engine-delta-overlay-on-forecast-baseline), [R-0016](research.md#r-0016--plan-scenario-versioning-immutable-snapshots-vs-editable-drafts), [R-0017](research.md#r-0017--plan-vs-ist-daily-computation--aggregation-grain), [R-0020](research.md#r-0020--grafana-dashboard-3-budgets-planistdeviation-provisioning)  
**Decisions:** **DEC-0073** (AE overlay-only compare delta); **DEC-0074** (AF 200 `no_active_plan`)  
**Sprint:** `/quick` **Q0019** (recommended)  
**Acceptance:** `docs/product/acceptance.md` rows **AD**, **AE**, **AF**  
**Spec-pack:** `docs/engineering/spec-pack/BUG-0011-{design-concept,crs,technical-specification}.md`  
**User guide:** `docs/user-guides/BUG-0011.md`  
**Related:** US-0004 DONE (plan engine); US-0014 OPEN (holistic UX epic — deferred); BUG-0004 superseded 404 note

**ID coordination:** US-0090 caveman compression forward-refs renumbered **DEC-0073 → DEC-0075** (runbook + scripts); BUG-0011 owns DEC-0073/DEC-0074.

### Symptom chain (frozen)

Operator on US-0010 external profile: `/planning` unusable — empty plan click no-op, Compare shows illogical negatives on zero-adjustment plans, Plan vs Actual tab broken by 404.

| Sub | Verdict | Root cause |
|-----|---------|------------|
| **AD** | CONFIRMED | No add-adjustment UI; first-run empty state Leasing-only; Custom Apply silent no-op |
| **AE** | CONFIRMED | `version_metrics` / `project_adjustments_in_memory` sum full `planned_net`, not overlay delta |
| **AF** | CONFIRMED | `NoActivePlan` → HTTP 404; `pvaQuery` no guided empty state (contrast risk-score 200 `no_score`) |

`isolation_scope`: artifact + repo source reads; no host `.env` / `.env_prod` secrets read.

### Sequencing (mandatory)

```text
BUG-0011
├── AE — DEC-0073 (backend compare metric, execute first)
│   ├── AE1 — monthly_overlay_delta_sum helper (overlay.rs / project.rs)
│   ├── AE2 — repository version_metrics + service in-memory path
│   └── AE3 — compare metric unit tests (zero overlay → 0.00)
├── AF — DEC-0074 (after AE1 helper frozen)
│   ├── AF1 — PlanVsActualApiResponse tagged enum; route 200 no_active_plan
│   └── AF2 — PVA tab guided empty state (mirror risk-score)
└── AD — execute (parallel after AF1 API contract frozen)
    ├── AD1 — first-run empty state + Create empty plan (POST template=custom)
    ├── AD2 — inline add/edit adjustment form (POST/PATCH)
    ├── AD3 — Custom Apply toast + query invalidation
    └── AD4 — compare help footnote + post-create Set active banner
→ T1 — integration tests (compare + plan-vs-actual)
→ V1 — operator OIDC /planning three-tab smoke
```

**Rule:** AE overlay helper before AF API shape freeze; AD PVA UX after AF1; Grafana Dashboard 3 **unchanged** (R-0020).

### AE — Overlay-only compare delta (DEC-0073)

#### Metric contract (frozen)

| Field | Formula | Empty plan |
|-------|---------|------------|
| `monthly_delta_sum` | Sum `build_overlay_deltas` for current month through `min(today, month_end)` | **0.00** when adjustments empty |
| `projected_month_end_balance` | Full scenario `planned_balance` at month-end horizon | May be negative (baseline forecast) — not zeroed |

**Files:** `backend/src/plan/{overlay,project,repository,service}.rs`.

**Endpoint scope:** `/compare` + React Compare tab only — not Grafana `budgets` panels.

#### Impact table (non-empty plans)

| Template | Before (bug) | After (correct) |
|----------|--------------|-----------------|
| Custom / Current, 0 lines | ~full forecast net | **0.00** delta |
| Leasing (+€300/mo) | baseline + leasing | **~-300/mo** overlay |
| Savings mode | baseline-dominated | net overlay (removals + cut) |

Release note mandatory — numbers shift for all plans (R-0016 alignment).

### AF — Plan-vs-actual empty API (DEC-0074)

#### API contract (frozen)

Mirror `RiskScoreApiResponse` pattern in `backend/src/api/plans.rs`:

```json
{ "status": "no_active_plan", "reason": "no_active_plan" }
```

HTTP **200** when no active plan; existing `ok` payload unchanged when active.

**Reject:** 404 via `plan_error_status`; auto-activate on create; 200 + empty `rows` only.

#### Frontend contract (frozen)

| Surface | Behavior |
|---------|----------|
| `pvaQuery` | `retry: false`; branch on `status` |
| `no_active_plan` | Guided card — create plan + Set active CTA |
| `ok` | Existing chart/table |

**Files:** `backend/src/api/plans.rs`, `backend/src/plan/types.rs`, `frontend/src/pages/PlanningPage.tsx`.

### AD — First-run + add-line UX (execute scope, no DEC)

| Gap | Fix |
|-----|-----|
| Empty state Leasing-only | Template card grid + **Create empty plan** (`POST { name, template: "custom" }`) |
| No POST wiring | Inline form above table → `add_adjustment` / `update_adjustment` |
| Custom Apply silent | Toast "Custom plan ready — add lines below" |
| Set active reminder | Inline banner after first create |

Bound to **US-0014** for wizard/tooltip polish — out of BUG-0011 scope.

### Codebase map (planning slice)

| Path | Role | BUG-0011 touch |
|------|------|----------------|
| `backend/src/plan/overlay.rs` | `build_overlay_deltas` | AE1 helper |
| `backend/src/plan/repository.rs` | `version_metrics`, `build_compare_metrics` | AE2 |
| `backend/src/plan/service.rs` | `plan_vs_actual`, `project_adjustments_in_memory` | AE2, AF1 |
| `backend/src/api/plans.rs` | routes, `plan_error_status`, `risk_score` pattern | AF1 |
| `frontend/src/pages/PlanningPage.tsx` | Scenarios / Compare / PVA tabs | AD1–AD4, AF2 |
| `grafana/provisioning/dashboards/analytics/budgets.json` | Dashboard 3 | **No change** |

### Task map (Q0019)

| Order | Task | Layer | Est. | Acceptance |
|-------|------|-------|------|------------|
| 1 | **AE1** overlay delta helper | backend plan | 2h | **AE** |
| 2 | **AE2** wire repository + service compare paths | backend plan | 2h | **AE** |
| 3 | **AE3** compare metric unit tests | backend tests | 2h | **AE** |
| 4 | **AF1** tagged PVA API 200 `no_active_plan` | backend API | 2h | **AF** |
| 5 | **AF2** PVA guided empty state | frontend | 2h | **AF** |
| 6 | **AD1** first-run Create empty plan | frontend | 2h | **AD** |
| 7 | **AD2** inline add/edit adjustment form | frontend | 3h | **AD** |
| 8 | **AD3** Custom Apply toast + invalidation | frontend | 1h | **AD** |
| 9 | **AD4** compare footnote + Set active banner | frontend | 1h | **AE**, **AD** |
| 10 | **T1** compare + PVA integration tests | backend tests | 2h | **AD/AE/AF** |
| 11 | **V1** operator OIDC `/planning` smoke | verify-work | 1h | footer |

**Count:** 11 tasks (< `SPRINT_MAX_TASKS` 12) → **`/quick` Q0019**; no split.

**Total estimate:** ~20h (dev ~19h + operator V1 ~1h).

### Test strategy

| Check | Type | Pass criteria |
|-------|------|---------------|
| AE3 | Unit | Zero adjustments → `monthly_delta_sum` = 0.00; Leasing ~-300 overlay |
| AF1 | Unit | `no_active_plan` serializes 200 tagged JSON |
| T1 | Integration | Compare endpoint + PVA route with/without active plan |
| AD2 | Manual/UI | POST adjustment creates row; table editable |
| Grafana | Regression | Dashboard 3 panels unchanged (no SQL edit) |
| V1 | Operator | `/planning` Scenarios + Compare + Plan vs Actual on OIDC deploy |

### Triad check (architecture phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#BUG-0011` | Discovery notes + research resolution linked | pass |
| `docs/product/acceptance.md` BUG-0011 | AD/AE/AF unchanged; mapped to tasks | pass |
| `backend/src/plan/*` + `api/plans.rs` | Root causes documented in codebase map | pass |
| `frontend/src/pages/PlanningPage.tsx` | AD/AF gaps documented | pass |
| R-0070 | Six questions resolved; DEC-0073/0074 recommended | pass |

`triad_hot_surface`: post-write `--check` required; architecture § BUG-0011 appended; decisions DEC-0073/DEC-0074 formalized.

### Decisions (BUG-0011)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0073 | AE compare metric | Overlay-only `monthly_delta_sum`; projected balance unchanged; shared helper |
| DEC-0074 | AF empty API | PVA 200 tagged `no_active_plan`; guided frontend; no auto-activate |

Full records: `decisions/DEC-0073.md`, `decisions/DEC-0074.md`

### Risks

| Risk | Mitigation |
|------|------------|
| Compare number shift (non-empty plans) | Release note; R-0016 intent |
| DEC-0073 ID collision (US-0090) | Renumbered to DEC-0075 in runbook/scripts |
| Negative projected balance on empty overlay | Help text; do not zero balance |
| PVA breaking change (404→200) | Changelog + user guide |
| Scope creep into US-0014 | AD bounded; epic deferred |

### Acceptance mapping

| Row | Architecture slice | Verify |
|-----|-------------------|--------|
| **AD** | AD1–AD3 | Create empty plan + add-line UX; not silent no-op |
| **AE** | AE1–AE3, AD4 | Zero/neutral compare deltas on empty plan |
| **AF** | AF1–AF2 | PVA 200 JSON; guided tab when no active plan |
| Footer | V1 | OIDC `/planning` three-tab regression |

### Next phase

**`/sprint-plan` Q0019** — materialize `sprints/quick/Q0019/task.json` from task table; AE-before-AF order frozen; then `/plan-verify` → `/execute`.

---

