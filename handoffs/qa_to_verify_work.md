# QA → Verify-work handoff — Q0029 / BUG-0021

**Bug:** BUG-0021  
**Quick task:** Q0029  
**QA verdict:** **PASS** (2026-06-11)  
**Orchestrator:** `auto-20260611-bug0021`  
**Decisions:** DEC-0110, DEC-0111  
**Next phase:** `/verify-work`

## QA summary

- **Blocking findings:** 0
- **Tasks verified PASS at qa:** EA1, EA2, EB1, EB2, EA3, T1, G1
- **Tasks deferred (expected):** V1 (operator gates)
- **Tests re-run:** `cargo test --lib` **213/213**; `bug0021_wealth_account_role` **4/4** (seed skipped — migration 015 checksum); `npm test` **9/9**; `npm run build` **PASS**

## Operator prerequisites (required before V1 live probes)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild backend + frontend with Q0029 changes (EA1–EB2 + EB1 SQL).
2. **SNAPSHOT_UPSERT_OR_SYNC** (optional) — Full sync or daily wealth snapshot upsert before BL snapshot/Grafana gate.

## BK/BL oracles — qa-stage vs verify-work

| Oracle | qa-stage | verify-work |
|--------|----------|-------------|
| **BK-FORECAST** | Static import + no Suspense on Monthly; build chunk audit PASS | Forecast → Monthly: no multi-second **Loading category filter…**; combobox ≤1 s |
| **BK-WEALTH** | Static import + no Suspense on Overview PASS | Wealth → Overview: same |
| **BL-API** | COALESCE SQL + mirror probe PASS; live API null pre-deploy | `GET /api/v1/wealth` — Giro/savings/cash `account_role` non-null (`defaultAsset`, `savingAsset`, `cashWalletAsset`) |
| **BL-UI** | `formatAccountRole` map PASS | Role column shows Checking / Savings / Cash wallet (not all em dash) |
| **BL-SNAPSHOT** | N/A at qa | `net_worth_snapshots.payload.accounts[*].account_role` non-null post-upsert |
| **BL-GRAFANA** | N/A at qa | Portfolio dashboard role column populated (optional) |
| **OIDC** | N/A at qa | `/forecast`, `/wealth`, `/api/v1/wealth` smoke on omniflow |

Live mirror at qa: COALESCE probe shows roles in DB; deployed `:18080` API returns null until rebuild.

## Acceptance row status (post-QA)

| Row | Code/static QA | Runtime (verify-work) |
|-----|----------------|------------------------|
| **BK** | PASS — DEC-0110 static import on Forecast/Wealth/Planning; CategoryTrendChart lazy unchanged | Pending deploy + browser smoke |
| **BL** | PASS — DEC-0111 COALESCE SQL + label map + mirror SQL probe | Pending deploy + API/UI/snapshot oracle |

## Verify-work focus

- Execute `sprints/quick/Q0029/uat.md` per acceptance rows **BK**, **BL**
- Post-deploy: confirm `GET /api/v1/wealth` roles non-null; Wealth Role column human labels
- Optional: snapshot/Grafana portfolio role column after upsert
- OIDC omniflow re-check on forecast/wealth routes

## Artifacts

- `sprints/quick/Q0029/qa-findings.md`
- `handoffs/qa_report.md`
- `decisions/DEC-0110.md`, `decisions/DEC-0111.md`
- `handoffs/dev_to_qa.md` (Q0029 top section)
- `sprints/quick/Q0029/progress.md`

`fresh_context_marker`: qa-20260611-bug0021-qa-fresh  
`runtime_proof_id`: runtime-proof-qa-20260611-bug0021-001  
`phase_boundary`: qa → verify-work

**Next:** `/verify-work` in fresh subagent/chat (role: qa). Do not begin verify-work in this subagent.
