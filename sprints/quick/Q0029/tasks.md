# Tasks — Q0029 (BUG-0021)

**Bug:** BUG-0021  
**Task count:** 7 mandatory + 1 optional P2 (8/12 under `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260611-q0029-bug0021`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **EA1** | Task **EA1** | ForecastPage static CategoryFilter; remove Suspense on Monthly tab |
| **EA2** | Task **EA2** | WealthPage static CategoryFilter; remove Suspense on Overview card |
| **EA3** | Task **EA3** | PlanningPage parity — optional P2 (out of BK) |
| **EB1** | Task **EB1** | `COALESCE(attributes, root)` SQL in `load_asset_accounts` |
| **EB2** | Task **EB2** | `formatAccountRole` label map on WealthPage Role column |
| **T1** | Task **T1** | `bug0021_wealth_account_role.rs` nested payload → role extract |
| Static gate | Task **G1** | `cargo test` + frontend build + blast-radius |
| BK/BL runtime | Task **V1** | verify-work after deploy |

## Execute order

```text
EA1 ∥ EA2 ∥ EB1
  → EB2 (WealthPage — coordinate with EA2)
  → EA3 (optional)
  → T1
  → G1
  → operator: BACKEND_FRONTEND_DEPLOY
  → operator: SNAPSHOT_UPSERT_OR_SYNC (optional)
  → V1 verify-work
```

**Parallelism:** EA1 (frontend), EA2 (frontend), EB1 (backend) are disjoint files — may
run in parallel. EB2 touches WealthPage (same file as EA2) — sequence after EA2 or
single commit. T1 blocked on EA1–EB2 (EA3 not required for T1).

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **BK** | EA1, EA2, T1, G1, V1 | No multi-second **Loading category filter…** on Forecast Monthly / Wealth Overview; combobox ≤1 s |
| **BL** | EB1, EB2, T1, G1, V1 | API `account_role` non-null; Role column labels; snapshot/Grafana post-upsert; OIDC smoke |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| EA1 | ForecastPage static CategoryFilter import | 1h | open | **BK** | P0 |
| EA2 | WealthPage static CategoryFilter import | 1h | open | **BK** | P0 |
| EB1 | wealth/repository.rs account_role SQL path | 1.5h | open | **BL** | P0 |
| EB2 | WealthPage formatAccountRole label map | 1h | open | **BL** | P0 |
| EA3 | PlanningPage CategoryFilter parity (optional) | 0.5h | open | — | P2 |
| T1 | Integration tests BK/BL + regression | 2h | open | **BK**, **BL** | P0 |
| G1 | Automated gate | 0.5h | open | **BK**, **BL** | P0 |
| V1 | verify-work operator smoke | 2h | open | **BK**, **BL** | P0 |

---

## EA1 — ForecastPage static CategoryFilter import

**Status:** open  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0021 **BK** — **DEC-0110**

### Description

In `frontend/src/pages/ForecastPage.tsx`:

1. Replace `React.lazy(() => import("../components/category/CategoryFilter")…)` with
   top-level `import { CategoryFilter } from "../components/category/CategoryFilter"`.
2. Remove `<Suspense fallback="Loading category filter…">` wrapper around CategoryFilter
   on the **Monthly** tab only.
3. **Unchanged:** `CategoryTrendChart` lazy + Suspense; other chart lazy boundaries.
4. **Opportunistic:** fix unused `hasForecast` TS6133 if present (BUG-0020 docker build blocker).

**Files:** `frontend/src/pages/ForecastPage.tsx`

### Done when

- [ ] CategoryFilter renders on Monthly tab without Suspense fallback on tab switch
- [ ] CategoryTrendChart lazy boundary preserved
- [ ] No TS6133 unused import blocking build

### Verification

Browser or trace: tab switch to Monthly — no **Loading category filter…** for multi-second duration.

---

## EA2 — WealthPage static CategoryFilter import

**Status:** open  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0021 **BK** — **DEC-0110**

### Description

In `frontend/src/pages/WealthPage.tsx`:

1. Replace lazy CategoryFilter import with static top-level import.
2. Remove `<Suspense>` wrapper around CategoryFilter in **Overview** → Category spending card.
3. **Unchanged:** `CategoryTrendChart` lazy + Suspense; `WealthChart` lazy boundary.

**Files:** `frontend/src/pages/WealthPage.tsx`

### Done when

- [ ] CategoryFilter on Overview renders without Suspense chunk fallback
- [ ] CategoryTrendChart lazy boundary preserved

### Verification

Wealth → Overview tab: combobox interactive ≤1 s; no sustained **Loading category filter…**.

---

## EB1 — wealth/repository.rs account_role SQL path

**Status:** open  
**Depends on:** —  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0021 **BL** — **DEC-0111**

### Description

In `backend/src/wealth/repository.rs` `load_asset_accounts`:

Replace root-only extract:

```sql
payload->>'account_role' AS account_role
```

With:

```sql
COALESCE(
  payload->'attributes'->>'account_role',
  payload->>'account_role'
) AS account_role
```

Update matching test SQL constant (~L133). **No change** to `active` /
`include_net_worth` root-level reads.

**Files:** `backend/src/wealth/repository.rs`

### Done when

- [ ] SQL uses COALESCE attributes/root path
- [ ] Test constant aligned
- [ ] `GET /api/v1/wealth` returns non-null `account_role` for seeded nested payload

### Verification

Post-deploy SQL probe per architecture § BUG-0021:

```sql
SELECT name,
       COALESCE(payload->'attributes'->>'account_role', payload->>'account_role') AS role
FROM accounts WHERE type = 'asset' ORDER BY name;
```

---

## EB2 — WealthPage formatAccountRole label map

**Status:** open  
**Depends on:** EB1 (API must return raw enum for meaningful labels)  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0021 **BL** — **DEC-0111**

### Description

Add frontend helper `formatAccountRole(raw: string | null | undefined): string`:

| Raw enum | UI label |
|----------|----------|
| `defaultAsset` | Checking |
| `cashWalletAsset` | Cash wallet |
| `savingAsset` | Savings |
| `sharedAsset` | Shared |
| `ccAsset` | Credit card |
| unknown enum | raw string |
| null/absent | — |

Replace `{row.account_role ?? "—"}` in Account breakdown **Role** column with
`formatAccountRole(row.account_role)`.

Optional: extract to `frontend/src/lib/accountRole.ts` for reuse.

**Files:** `frontend/src/pages/WealthPage.tsx`, optionally `frontend/src/lib/accountRole.ts`

### Done when

- [ ] Role column shows human labels for known Firefly enums
- [ ] Unknown enum shows raw value; null shows em dash
- [ ] API continues returning raw enum (no DTO label field)

### Verification

Wealth → Overview → Account breakdown: Cash wallet / Giro / savings show labels not all em dash.

---

## EA3 — PlanningPage CategoryFilter parity (optional)

**Status:** open  
**Depends on:** —  
**Estimate:** 0.5h  
**Acceptance hook:** — (P2 consistency; out of BK acceptance)

### Description

Apply same static CategoryFilter import policy to `frontend/src/pages/PlanningPage.tsx`
(L31–32, L854–855 lazy pattern). Remove Suspense wrapper around CategoryFilter.

**Files:** `frontend/src/pages/PlanningPage.tsx`

### Done when

- [ ] Planning page CategoryFilter uses static import
- [ ] No regression on Planning category filter UX

### Verification

Planning page: category filter renders without multi-second Suspense fallback.

---

## T1 — Integration tests BK/BL + regression

**Status:** open  
**Depends on:** EA1, EA2, EB1, EB2  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0021 **BK**, **BL**

### Description

Add `backend/tests/bug0021_wealth_account_role.rs` covering:

1. **BL** — seed account with nested `payload.attributes.account_role` →
   `load_asset_accounts` / `GET /api/v1/wealth` returns expected raw enum
   (`cashWalletAsset`, `defaultAsset`, `savingAsset`).
2. **BL** — root-only fallback path when attributes absent.
3. **Regression** — wealth list shape unchanged; categories endpoint smoke.

Frontend BK (no Suspense fallback) is primarily browser/V1 gated; optional Vitest
smoke if existing page test harness supports it.

**Files:** `backend/tests/bug0021_wealth_account_role.rs`

### Done when

- [ ] Nested attributes.account_role seed → non-null API field
- [ ] COALESCE root fallback covered
- [ ] No regression in existing wealth integration tests

### Verification

`cargo test --test bug0021_wealth_account_role` → all PASS.

---

## G1 — Automated gate

**Status:** open  
**Depends on:** T1  
**Estimate:** 0.5h  
**Acceptance hook:** DEC-0110/0111 verification gates — static/automated

### Description

Run and record automated checks in `sprints/quick/Q0029/progress.md`:

1. `cargo test --test bug0021_wealth_account_role` → PASS.
2. Related wealth test suites unchanged.
3. `npm run build` (frontend) → PASS (confirms TS6133 resolved if fixed in EA1).
4. `git diff --stat` blast radius matches frozen file list.

**Files:** `sprints/quick/Q0029/progress.md`

### Done when

- [ ] All automated checks PASS, recorded in progress.md
- [ ] No forbidden paths touched (CategoryFilter logic, firefly sync, Grafana JSON)

### Verification

Test output pasted in progress.md; diff stat confirms scope.

---

## V1 — verify-work operator smoke

**Status:** open  
**Depends on:** G1 + operator BACKEND_FRONTEND_DEPLOY  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0021 **BK**, **BL**

### Description

Populate `sprints/quick/Q0029/uat.md` and `uat.json` after deploy on
localhost:18080 (and optional omniflow OIDC):

1. **BK-FORECAST** — Forecast → Monthly: no multi-second **Loading category filter…**; combobox ≤1 s.
2. **BK-WEALTH** — Wealth → Overview: same.
3. **BL-API** — `GET /api/v1/wealth` — asset accounts `account_role` non-null.
4. **BL-UI** — Account breakdown Role column human labels.
5. **BL-SNAPSHOT** — latest `net_worth_snapshots.payload.accounts` carries `account_role` (post-upsert).
6. **BL-GRAFANA** — portfolio dashboard role column populated (optional post-sync).
7. **OIDC** — `/forecast`, `/wealth`, `/api/v1/wealth` smoke on omniflow profile.

**Files:** `sprints/quick/Q0029/uat.md`, `sprints/quick/Q0029/uat.json`

### Done when

- [ ] Rows **BK**, **BL** probed per acceptance.md matrix
- [ ] Regression gates documented
- [ ] `uat.md` and `uat.json` populated with results

**Operator gates:** **BACKEND_FRONTEND_DEPLOY**; optional **SNAPSHOT_UPSERT_OR_SYNC**.
