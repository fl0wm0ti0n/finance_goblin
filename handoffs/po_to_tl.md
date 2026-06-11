# discovery-20260611-bug0021 — BUG-0021 Frontend UX polish (category filter delay, wealth role column)

**From:** PO **To:** Tech Lead **Bug:** BUG-0021 **Run:** `auto-20260610-bug0019`
**Date:** 2026-06-11 **Next phase:** `/research` (role: tech-lead)
**Intake evidence (read-only):** `handoffs/intake_evidence/intake-20260609-frontend-ux.json` (`intake-20260609-frontend-ux`, `small-intake-pack`, kind=bug)
**Prior context:** BUG-0020 released Q0028 (`bug0020-q0028`, DEC-0109); bug queue continues; operator deferred ForecastPage TS6133 rebuild from BUG-0020 release notes.

## Bug summary

Post-US-0020 rebuild (localhost:18080), two P3 frontend polish gaps from UI audit 2026-06-09:

- **EA (UI-011):** Forecast → **Monthly** or Wealth → **Overview** — **Loading category filter…** Suspense fallback visible **3–5s** before combobox renders.
- **EB (UI-012):** Wealth → **Overview** → Account breakdown — **Role** column shows **—** for every account (Cash wallet, Giro, savings).

## Acceptance rows (canonical: `docs/product/acceptance.md` BUG-0021)

- **(BK)** CategoryFilter on Forecast Monthly and Wealth Overview becomes interactive within **~1s** of tab visit—not multi-second **Loading category filter…** Suspense fallback under normal local/omniflow load. *(= sub-defect EA)*
- **(BL)** Wealth account breakdown **Role** column shows Firefly account role/type when mirror provides it, or column is hidden/documented when unsupported—not permanent em dash for all rows. OIDC-enabled deploy regression checks pass. *(= sub-defect EB)*

## Reproduction / evidence pointers

| Ref | Surface | Pointer |
|-----|---------|---------|
| UI steps | EA | Forecast → Monthly or Wealth → Overview — `handoffs/intake_evidence/ui-audit-20260609-local.json` **UI-011** |
| UI steps | EB | Wealth → Overview → Account breakdown — **UI-012**, screenshot `wealth-page.png` |
| Lazy import | EA | `frontend/src/pages/ForecastPage.tsx` L20–22 `lazy(() => import("../components/category/CategoryFilter")…)`; monthly tab L276–282 Suspense fallback |
| Lazy import | EA | `frontend/src/pages/WealthPage.tsx` L15–17, L179–181 (overview Category spending card) |
| Component | EA | `frontend/src/components/category/CategoryFilter.tsx` — lightweight select/combobox; `useQuery` → `fetchCategories`; **no ECharts** |
| Separate lazy | EA | `CategoryTrendChart` lazy in same pages (ECharts) — loads **after** category selection, not the filter fallback |
| Out-of-scope twin | EA | `frontend/src/pages/PlanningPage.tsx` L31–32, L854–855 — same lazy CategoryFilter pattern (not in BK acceptance) |
| UI render | EB | `frontend/src/pages/WealthPage.tsx` L220 `{row.account_role ?? "—"}` |
| API type | EB | `frontend/src/lib/api.ts` `account_role?: string \| null` on wealth account row |
| API assembly | EB | `backend/src/wealth/service.rs` L80–102 maps `a.account_role` from repository row |
| SQL extract | EB | `backend/src/wealth/repository.rs` L31 `payload->>'account_role' AS account_role` (also L133 snapshot query) |
| Sync ingest | EB | `backend/src/firefly/mod.rs` L249–287 — stores full Firefly API `item` in `accounts.payload`; logs `attrs["account_role"]` from **attributes** |
| Contrast | EB | `backend/src/plan/repository.rs` L752 uses `t.payload->'attributes'->>'type'` for transactions — nested path pattern |
| Research spec | EB | `docs/engineering/research.md` R-0001 wealth SQL documents same root-level `payload->>'account_role'` |
| Snapshots | EB | `backend/src/wealth/service.rs` L271–283 snapshot `accounts` array inherits null `account_role` from query |
| Grafana | EB | `grafana/provisioning/dashboards/analytics/portfolio.json` — `elem->>'account_role'` from snapshot payload |

## Discovery observations (facts only — root cause is research scope)

### EA — CategoryFilter lazy-load delay

1. **Suspense is chunk-bound:** Fallback text **Loading category filter…** is shown until the `React.lazy` dynamic import resolves—not until `fetchCategories` returns. CategoryFilter itself is small (~96 LOC, react-query + API helper only).
2. **Acceptance surfaces match code:** BK scopes Forecast **Monthly** tab and Wealth **Overview**; both wrap CategoryFilter in `Suspense` with the reported fallback string.
3. **ECharts not in filter chunk:** `CategoryTrendChart` (imports `echarts-for-react`) is a **separate** lazy boundary; it mounts only when `categoryId` is set—unlikely cause of filter delay.
4. **Likely fix axis (research):** Eager/static import of CategoryFilter, route-level prefetch of the chunk, or a non-blocking skeleton—balance against P3 “avoid material bundle regression” (intake constraint).
5. **PlanningPage parity:** Same lazy pattern exists on Planning (out of BK) — research should decide whether fix is page-local or shared component import policy.

### EB — Wealth Role column empty

1. **UI contract is pass-through:** Frontend correctly renders API `account_role` or em dash; no missing column component.
2. **SQL path mismatch (strong hypothesis):** Repository reads `payload->>'account_role'` at JSON **root**, but Firefly sync persists the full API **item** (`upsert_account(…, item)`) where `account_role` lives under `attributes` (sync logs `attrs["account_role"]`).
3. **Filters may mask bug:** `active` / `include_net_worth` use same root-level `payload->>'…'` with `COALESCE(…, true)` — accounts still appear when attributes are nested.
4. **Downstream propagation:** Null `account_role` flows into `GET /api/v1/wealth`, daily snapshot payload, and Grafana portfolio SQL (`elem->>'account_role'`).
5. **Tests blind spot:** Integration seeds often use `payload: '{}'` — would not catch nested-path extraction gap.
6. **Display vs raw enum:** Firefly roles (`defaultAsset`, `cashWalletAsset`, `savingAsset`, etc. per R-0001) may need human labels for BL “useful” column—or hide when truly absent.

## Open questions for research

1. **EA timing proof:** On localhost:18080 / omniflow, measure lazy-chunk load vs categories API—confirm Suspense duration tracks JS import not network.
2. **EA fix choice:** Static import vs `import()` prefetch on route enter vs slimmer suspense UX—which satisfies BK ≤1s without violating bundle constraint?
3. **EA scope:** Include PlanningPage lazy CategoryFilter in fix for consistency?
4. **EB live payload:** Sample `accounts.payload` for Giro / savings / cash wallet—verify `attributes.account_role` present post-sync.
5. **EB fix axis:** SQL `payload->'attributes'->>'account_role'` vs denormalize at sync vs Rust mapping layer—which is canonical and snapshot-safe?
6. **EB display contract:** Map Firefly enums to operator labels, show raw enum, or hide column when null after fix?
7. **EB Grafana:** Does portfolio dashboard role column need the same backfill/re-snapshot pass?
8. **Regression gates:** Smoke BK (combobox ≤1s on tab switch), BL (non-dash role when mirror has metadata), OIDC deploy template per acceptance.

## Done definition (intake)

Operator perceives snappy category filter; Wealth breakdown Role column useful or removed.

## Recommended next phase

`/research` (role: tech-lead) — confirm EA chunk-load hypothesis with timing probes; confirm EB JSON path on live mirror; decide fix axes and regression scope; then architecture.

---

## Prior segment hot pointers (archived)

- discovery-20260610-bug0020 — **DONE** (released `bug0020-q0028`) — BI/BJ subscriptions list · DEC-0109 · R-0090
- refresh-context-20260611-bug0020 — **DONE** — `handoffs/curator_refresh.md` · bug queue advance to BUG-0021
- discovery-20260610-bug0019 — **DONE** (released `bug0019-q0027`) — Grafana CA/CB → BG/BH · DEC-0108 · R-0089

`triad_hot_surface`: prior BUG-0020 discovery body retained in archive refs; BUG-0021 discovery written fresh

---

## Research findings (2026-06-11) — R-0091

**Phase:** research (tech-lead) · **Orchestrator:** `auto-20260611-bug0021`

### EA — CategoryFilter delay (BK)

- **Confirmed:** Suspense fallback is `React.lazy` dynamic import resolution, **not** `fetchCategories` (API 2–5 ms on :18080).
- **Chunk:** `CategoryFilter-B6dgjo-5.js` = 1.5 KB; direct fetch ~1 ms; depends on already-loaded main split — delay is lazy waterfall on tab switch, not network-heavy categories call.
- **Recommended fix axis:** **static import** on ForecastPage + WealthPage (negligible bundle cost); PlanningPage parity optional (out of BK).

### EB — Wealth Role column (BL)

- **Confirmed:** `payload->>'account_role'` null for 3/3 asset accounts; `payload->'attributes'->>'account_role'` populated (`cashWalletAsset`, `defaultAsset`, `savingAsset`); API returns `account_role: null` for all.
- **Root cause:** repository SQL root path vs Firefly full-item sync storage under `attributes`.
- **Recommended fix axis:** SQL `payload->'attributes'->>'account_role'` in `wealth/repository.rs`; optional human label map for BL; snapshots/Grafana heal on next daily upsert.

**Next phase:** `/architecture` (role: tech-lead) — DEC for EA import policy + EB display contract; map to quick sprint.

---

## Architecture findings (2026-06-11) — DEC-0110, DEC-0111

**Phase:** architecture (tech-lead) · **Orchestrator:** `auto-20260611-bug0021`

### EA — CategoryFilter (BK) — DEC-0110

- **Frozen:** static top-level import on `ForecastPage` (Monthly tab) and `WealthPage` (Overview category card); remove Suspense wrapper around CategoryFilter on those surfaces.
- **Unchanged:** `CategoryTrendChart` lazy + Suspense; other chart lazy boundaries.
- **Optional P2:** `PlanningPage` parity (out of BK).

### EB — Wealth Role column (BL) — DEC-0111

- **Frozen:** `COALESCE(payload->'attributes'->>'account_role', payload->>'account_role')` in `wealth/repository.rs` `load_asset_accounts` (+ test SQL constant).
- **Frozen:** frontend `formatAccountRole` human label map on WealthPage Role column; API returns raw enum; unknown → raw string; null → em dash.
- **Propagation:** snapshots + Grafana portfolio panel heal on next daily upsert — no Grafana JSON edit.

**Next phase:** `/sprint-plan` (role: tech-lead) — materialize quick sprint from architecture § BUG-0021 task tree.
