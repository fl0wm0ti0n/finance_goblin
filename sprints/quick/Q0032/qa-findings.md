# QA Findings — Quick Q0032 / BUG-0026

**Work item:** BUG-0026 (defect)  
**Quick task:** Q0032  
**QA phase:** `/qa`  
**Date:** 2026-06-13  
**Orchestrator:** `auto-20260613-bug0026`  
**Decisions:** DEC-0089 (extends US-0002 / US-0018; GATE-DEC-1 no new DEC)  
**QA agent:** fresh subagent (`qa-20260613-bug0026-qa-fresh`)

## Verdict

**PASS** — Independent re-run confirms BZ/CA implementation under frozen architecture gates **GATE-MONTH-1** / **GATE-LABEL-1** / **GATE-SCOPE-1** / **GATE-TEST-1** / **GATE-DEC-1**: `resolveForecastSummaryPoint` skips partial zero-income head (June 0 → July **3266.16**); shared subtitle **"Forecast for July 2026"** above four cards; cards and chart both use unfiltered `monthlyQuery` series per **DEC-0089**. Automated gates **24/24** npm (+7 `forecastSummaryMonth`), **PASS** build. **V1** runtime BZ/CA smoke on account **114** deferred to verify-work — pass-with-prerequisites (**FRONTEND_DEPLOY**).

**Blockers:** 0

## Scope

**Inputs reviewed:** `handoffs/dev_to_qa.md` (Q0032 top section), `sprints/quick/Q0032/{summary,progress,tasks,uat}.md`, `docs/product/acceptance.md` BUG-0026 row (BZ/CA), `docs/engineering/architecture.md` § BUG-0026, `frontend/src/pages/forecastSummaryMonth.ts`, `frontend/src/pages/forecastSummaryMonth.test.ts`, `frontend/src/pages/ForecastPage.tsx`. No host `.env`/secret files read. `cargo test --lib` not in blast radius (frontend-only per GATE-SCOPE-1).

## Test plan

| # | Test | Command / method | Result |
|---|------|-------------------|--------|
| T-1 | Frontend unit suite | `npm test` | **PASS** — 24/24 (4.33s) |
| T-2 | Frontend build | `npm run build` | **PASS** — tsc + vite build (16.67s) |
| T-3 | GATE-MONTH-1 partial-month trap | `forecastSummaryMonth.test.ts` partialMonthTrap | **PASS** — resolves July **3266.16** |
| T-4 | GATE-MONTH-1 edge cases | `forecastSummaryMonth.test.ts` (5 resolve cases) | **PASS** — positive head, all-zero, single-month, empty |
| T-5 | GATE-LABEL-1 subtitle | `formatForecastSummarySubtitle("2026-07-01")` | **PASS** — **"Forecast for July 2026"** |
| T-6 | F1 ForecastPage wire | Code review L152–155, L315–345 | **PASS** — useMemo + subtitle above grid; card values from resolved point |
| T-7 | DEC-0089 category filter | `monthlyQuery` key L100–107; CategoryFilter L276–284 | **PASS** — no `categoryId` in query key; helper text unchanged |
| T-8 | GATE-SCOPE-1 blast radius | File scope vs architecture § codebase map | **PASS** — 3 frontend files only; no `MonthlyChart.tsx`, backend, API |
| T-9 | Chart parity | `MonthlyChart series={monthlyQuery.data?.series ?? []}` L349 | **PASS** — full unfiltered series; cards use resolved point from same series |
| T-10 | User-visible metadata guard | `python3 scripts/check-user-visible-metadata.py` | **skipped** — entrypoint missing (`METADATA_SANITIZATION_POLICY_MISSING`); manual review of changed strings shows no internal metadata tokens |
| T-11 | V1 operator smoke | FRONTEND_DEPLOY + `/forecast` Monthly account **114** | **DEFERRED** — verify-work |

### T-1 output

```
Test Files  6 passed (6)
Tests       24 passed (24)
  forecastSummaryMonth.test.ts  7 passed
  planSelector.test.ts          8 passed
  planningFeedback.test.ts      4 passed
  GoalStatsStrip.test.tsx       2 passed
  CategoryTrendChart.test.tsx   1 passed
  ChatPanel.test.tsx            2 passed
```

## Generated baseline test evidence (US-0066)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | node (vitest) |
| `generated_test_command` | `npm test` |
| `generated_test_result` | pass |
| `generated_test_output_ref` | T-1 output above |
| `generated_test_paths_ref` | `frontend/src/pages/forecastSummaryMonth.test.ts` |
| `generated_test_reason_code` | (none) |

## Code review vs decisions

| Gate / Decision | Contract | Review |
|-----------------|----------|--------|
| **GATE-MONTH-1** | Skip partial zero-income head; first income month | **PASS** — `resolveForecastSummaryPoint` L13–15 matches frozen algorithm; `?? series[0]` fallback aligns edge-case table (all-zero → series[0]) |
| **GATE-LABEL-1** | Shared subtitle above card grid | **PASS** — `formatForecastSummarySubtitle` L27–28; rendered L325 above `.grid` |
| **GATE-SCOPE-1** | Frontend-only; no API/backend/chart edits | **PASS** — new helper + ForecastPage wire only |
| **GATE-TEST-1** | Vitest pure helper + partial-month fixture | **PASS** — 7/7 cases; partialMonthTrap reproduces account 114 pattern |
| **GATE-DEC-1** | No new DEC | **PASS** — contract documented in architecture § BUG-0026 only |
| **DEC-0089** | Category filter scopes trend only; cards use unfiltered monthly series | **PASS** — `monthlyQuery` key unchanged; cards/chart independent of `categoryId` |

## Acceptance row status (qa-stage)

| Row | qa-stage evidence | Status |
|-----|-------------------|--------|
| **BZ** | Partial-month trap resolves July **3266.16**; cards bind resolved point; chart plots full series from same API payload | **PASS** at qa — live account **114** browser smoke deferred V1 |
| **CA** | Subtitle **"Forecast for July 2026"** via vitest + ForecastPage render path | **PASS** at qa — operator-visible label deferred V1 |

## Task verdict matrix

| Task | Status (execute) | QA verdict | Notes |
|------|------------------|------------|-------|
| H1 | done | **PASS** | `forecastSummaryMonth.ts` exports match architecture contract |
| F1 | done | **PASS** | useMemo + subtitle + card values from resolved point |
| T1 | done | **PASS** | 7/7 vitest |
| G1 | done | **PASS** | Independent 24/24 + build PASS |
| V1 | deferred | **DEFERRED** | FRONTEND_DEPLOY required |

## Non-blocking notes (carry to verify-work)

- **AI-mapped callout** still keys off `series[0]?.ai_mapped` (L286) while summary cards use resolved month — pre-existing pattern; optional P2 alignment if callout should follow resolved month.
- V1 requires operator **FRONTEND_DEPLOY** (frontend rebuild only; no migration) before `/forecast` Monthly smoke on account **114**.
- Implementation files remain uncommitted per sprint policy.

## Handoff

- **Next phase:** `/verify-work` (role: qa)
- **No return items** — `handoffs/qa_to_dev.md` not written (PASS; 0 blockers)

`fresh_context_marker`: qa-20260613-bug0026-qa-fresh  
`runtime_proof_id`: runtime-proof-qa-20260613-bug0026-001  
`phase_boundary`: qa → verify-work
