# Architecture archive pack (2026-06-11)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 12
- First archived heading: `# BUG-0021 — Frontend UX polish (category filter delay, wealth role column)`
- Last archived heading: `# BUG-0021 — Frontend UX polish (category filter delay, wealth role column)`
- Verification tuple (mandatory):
  - archived_body_lines=120
  - preamble_lines=10
  - retained_body_lines=2996

---

# BUG-0021 — Frontend UX polish (category filter delay, wealth role column)

**Status:** architecture complete (2026-06-11)  
**Discovery:** `discovery-20260611-bug0021` in `handoffs/po_to_tl.md`  
**Research:** [R-0091](research.md#r-0091--bug-0021-categoryfilter-lazy-load-delay--wealth-role-column-empty)  
**Decisions:** **DEC-0110** (CategoryFilter static import on BK surfaces); **DEC-0111** (wealth `account_role` attributes path + display labels)  
**Acceptance:** `docs/product/acceptance.md` rows **BK** (EA), **BL** (EB)

### Root cause (frozen, R-0091)

| Sub-defect | Symptom | Mechanism |
|------------|---------|-----------|
| **EA** | **Loading category filter…** 3–5 s on Forecast Monthly / Wealth Overview | `React.lazy` CategoryFilter + `<Suspense>` on tab render — fallback until dynamic `import()` resolves; categories API 2–5 ms; chunk ~1.5 KB. Not `fetchCategories` network wait. |
| **EB** | Role column em dash for all accounts | Repository SQL `payload->>'account_role'` at JSON root; Firefly sync stores full item with role under `attributes.account_role`. API/snapshot/Grafana inherit null. |

`isolation_scope`: artifact + repo source reads; read-only DB/API/chunk probes per R-0091; no host `.env` / secrets read.

### Architecture contract

```text
BUG-0021
├── EA1 — static CategoryFilter import (P0, DEC-0110)
│   └── ForecastPage.tsx — Monthly tab: top-level import; remove Suspense around CategoryFilter
├── EA2 — static CategoryFilter import (P0, DEC-0110)
│   └── WealthPage.tsx — Overview category card: top-level import; remove Suspense around CategoryFilter
├── EA3 — PlanningPage parity (P2, optional)
│   └── PlanningPage.tsx — same static import for consistency (out of BK acceptance)
├── EB1 — SQL attributes path (P0, DEC-0111)
│   └── wealth/repository.rs — COALESCE(attributes, root) account_role extract
├── EB2 — Role display labels (P0, DEC-0111)
│   └── WealthPage.tsx (+ optional frontend/src/lib/accountRole.ts) — formatAccountRole helper
└── T1 — integration test + V1 operator smoke (P0)
    └── BK/BL gates; snapshot/Grafana propagation; OIDC regression
```

### Files (frozen)

| Path | Role | Touch |
|------|------|-------|
| `frontend/src/pages/ForecastPage.tsx` | Remove lazy CategoryFilter; static import on Monthly tab | **EA1** |
| `frontend/src/pages/WealthPage.tsx` | Remove lazy CategoryFilter; `formatAccountRole` on Role column | **EA2**, **EB2** |
| `frontend/src/pages/PlanningPage.tsx` | Optional static import parity | **EA3** |
| `frontend/src/lib/accountRole.ts` (new, optional) | Shared `ACCOUNT_ROLE_LABELS` + `formatAccountRole` | **EB2** |
| `backend/src/wealth/repository.rs` | `load_asset_accounts` SQL path + test constant L133 | **EB1** |
| `backend/tests/bug0021_wealth_account_role.rs` (new, recommended) | Seed nested payload → role extract | **T1** |

**No change:** `CategoryFilter.tsx` component logic; `CategoryTrendChart` lazy boundary; `firefly/mod.rs` sync storage; `grafana/provisioning/dashboards/analytics/portfolio.json` SQL shape; `GET /api/v1/categories` contract.

### SQL contract (EB1)

Replace root-only extract in `load_asset_accounts`:

```sql
COALESCE(
  payload->'attributes'->>'account_role',
  payload->>'account_role'
) AS account_role
```

`active` / `include_net_worth` root-level reads remain unchanged (out of scope — COALESCE true masks gap).

### Display contract (EB2)

| Raw enum (`account_role` API) | UI label |
|-------------------------------|----------|
| `defaultAsset` | Checking |
| `cashWalletAsset` | Cash wallet |
| `savingAsset` | Savings |
| `sharedAsset` | Shared |
| `ccAsset` | Credit card |
| unknown enum | raw string |
| null | — |

### Snapshot / Grafana propagation

1. `wealth/service.rs` maps `a.account_role` from repository row into API DTO (unchanged assembly).
2. `upsert_daily_snapshot` writes `accounts` array with fixed `account_role` — heals on next daily upsert post-deploy.
3. Grafana `portfolio.json` reads `elem->>'account_role'` from latest snapshot — no provisioning edit required.

Operator optional: trigger Full sync or wait for scheduler before BL snapshot gate.

### Verification gates

| Gate | Proof |
|------|-------|
| **BK** | Forecast → Monthly and Wealth → Overview: no multi-second **Loading category filter…**; combobox interactive ≤1 s |
| **BL API** | `GET /api/v1/wealth` — Giro/savings/cash wallet `account_role` non-null (`defaultAsset`, `savingAsset`, `cashWalletAsset`) |
| **BL UI** | Wealth Account breakdown Role shows human labels not em dash |
| **BL snapshot** | Latest `net_worth_snapshots.payload.accounts` carries `account_role`; Grafana portfolio role column populated |
| **T1** | Integration test: nested `attributes.account_role` seed → repository returns role |
| **OIDC** | `/forecast`, `/wealth`, `/api/v1/wealth` smoke on omniflow profile |

**Post-fix SQL probe:**

```sql
SELECT name,
       COALESCE(payload->'attributes'->>'account_role', payload->>'account_role') AS role
FROM accounts WHERE type = 'asset' ORDER BY name;
```

### Risks

| Risk | Mitigation |
|------|------------|
| Static import grows page chunks | ~1.5 KB — negligible; monitor build |
| Snapshot lag until daily upsert | UAT documents; optional manual sync |
| Incomplete label map | Fallback to raw enum |
| ForecastPage TS6133 from BUG-0020 | Operator prerequisite for docker build — fix before BACKEND_FRONTEND_DEPLOY |
| Root `active`/`include_net_worth` path hygiene | Out of scope; future bug if filters misbehave |

### Rollback

`git revert` frontend import/label changes + repository SQL. No migration. Snapshots retain last role values until next upsert.

### Next phase

`/sprint-plan` — materialize `/quick` sprint from task table (recommended ≤7 tasks under SPRINT_MAX_TASKS); then `/plan-verify` → `/execute`.

---

