# Plan-verify — Quick Q0009 / BUG-0003

**Sprint:** Q0009 (`/quick`)  
**Bug:** BUG-0003 — Omniflow production API 500 cascade, Bitunix test, Grafana SQL  
**Verified at:** 2026-06-05T18:00:00Z  
**Role:** QA  
**Verdict:** **PASS**

## Inputs

| Source | Path |
|--------|------|
| Acceptance | `docs/product/acceptance.md` — BUG-0003 rows **(F)**, **(G)**, **(H)** |
| Tasks | `sprints/quick/Q0009/tasks.md`, `task.json` |
| Sprint plan | `sprints/quick/Q0009/sprint.md` |
| Architecture | `docs/engineering/architecture.md` § BUG-0003 |
| Handoff | `handoffs/tl_to_dev.md` (`architecture-20260605-bug0003`) |

## Test plan (coverage review)

For each acceptance row, confirm at least one task with explicit done-when checks and feasible deploy/execute order.

| Row | Criterion (abbrev.) | Tasks | Covered |
|-----|---------------------|-------|---------|
| **(F)** | GET `/api/v1/*` **200** &lt;2s; settings `database_host: postgres`, `database_mode: external` | F1, F2 | yes |
| **(G)** | `POST …/exchanges/bitunix/test` not **400** unknown exchange; **200** or documented auth failure | G1, G2? | yes |
| **(H)** | Grafana `ds/query` **200**; datasource reaches in-network `postgres` | F1 (H1) | yes |
| Regression | OIDC + bundled-firefly footer | verify-work | yes (advisory) |

### Architecture contract alignment

| Slice | Contract (frozen) | Task(s) | Aligned |
|-------|-------------------|---------|---------|
| **F** | `DATABASE_HOST=postgres`; recreate app + grafana; GET APIs **200** | F1 | yes |
| **F** | Omniflow env guard in `.env.example` + runbook | F2 | yes |
| **G** | `ExchangeService::new` uses `effective_enabled()` for all connectors | G1 | yes |
| **G** | R-0058 futures auth spike only if auth-fails after G1+F1 | G2 | yes (gated) |
| **H** | H1 = F1 smoke on `ds/query` **200** | F1 | yes |

**No new DEC.** Extends **DEC-0056** (external Postgres topology); **R-0052** (host naming); **R-0058** gated for G2.

Frozen boundaries (no Traefik changes, no BUG-0002 merge, no compose hardcode, G2 gate, H2 deferred) appear in `sprint.md`, `task.json`, and architecture § BUG-0003.

### Task traceability (F1–F2, G1–G2)

| Task | Acceptance hook | Orphan |
|------|-----------------|--------|
| F1 | **(F)(H)** operator — postgres host, APIs, Grafana SQL | no |
| F2 | **(F)** guardrail — docs/runbook | no |
| G1 | **(G)** code — registry parity | no |
| G2 | **(G)** conditional — R-0058 auth | no |

H1 is embedded in F1 tests per `task.json`; H2 explicitly out of sprint scope.

### Dependency review

| Check | Result |
|-------|--------|
| Circular dependencies | none |
| Execution order feasible | yes — F2 ∥ G1 → deploy → F1 → smoke → G2 if gate |
| G2 after G1 + F1 | `depends_on` + gate in task.json |
| F1 before F/H acceptance | documented in deploy_order |

### Test coverage review

| Layer | Task | Scope |
|-------|------|-------|
| Operator | F1 | settings host; sample GETs; grafana ds/query |
| Doc review | F2 | omniflow block + runbook symptom table |
| Unit | G1 | configured + TOML disabled → connector in `new()` |
| Operator (gated) | G2 | bitunix test auth body documented |
| UAT (post-execute) | verify-work | Rows F/G/H + regression footer |

## Findings

### Gaps

None.

### Orphan tasks

None (4/4 tasks map to rows F, G, or H; G2 gated).

### Advisories (non-blocking)

| ID | Note |
|----|------|
| ADV-1 | Regression footer — operator verify-work |
| ADV-2 | Rows **(F)**/**(H)** runtime requires F1 on omniflow |
| ADV-3 | G2 likely skipped if G1 fixes registry-only failure |
| ADV-4 | BUG-0002/Q0008 parallel — do not merge |
| ADV-5 | Representative GET probes sufficient for row **(F)** |

## Verdict

**PASS** — Plan is ready for `/execute`. Machine-readable record: `sprints/quick/Q0009/plan-verify.json`.
