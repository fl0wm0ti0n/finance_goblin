# UAT — Q0009 (BUG-0003)

**Status:** POPULATED — verify-work **PASS** (2026-06-05 re-run)  
**Acceptance:** `docs/product/acceptance.md` — BUG-0003 rows **(F)**, **(G)**, **(H)** (checkbox **unchecked** — pending `/release`)  
**Plan-verify:** `sprints/quick/Q0009/plan-verify.json` — **PASS** (2026-06-05)  
**Verify-work:** `sprints/quick/Q0009/verify-work-findings.md` — **PASS**

| Row | Task(s) | Verify-work result |
|-----|---------|-------------------|
| **(F)** | F1, F2 | **PASS** — `database_host: postgres`; GET APIs **200** &lt;0.1s |
| **(G)** | G1, G2? | **PASS** — bitunix test **200** `Spot balance read OK` |
| **(H)** | F1 | **PASS** — Grafana ds/query **200** `SELECT 1` |
| Regression | post-F1 | **DEFERRED** — OIDC browser advisory |

## Operator smoke checklist (verify-work)

### F — DB host (F1 + F2 guardrail)

| # | Check | Pass |
|---|-------|------|
| 1 | `GET /api/v1/settings` → `database_host: postgres`, `database_mode: external` | **yes** |
| 2 | `GET /api/v1/alerts/unread-count` **200** &lt;2s | **yes** (~74ms) |
| 3 | `GET /api/v1/sync/entities` **200** &lt;2s | **yes** (~73ms) |
| 4 | `GET /api/v1/exchanges` **200** &lt;2s | **yes** (~63ms) |
| 5 | `.env.example` omniflow block warns against `host.docker.internal` on external | **yes** (static F2) |

### G — Bitunix (G1; G2 if gated)

| # | Check | Pass |
|---|-------|------|
| 6 | `POST /api/v1/exchanges/bitunix/test` — not **400** `unknown exchange: bitunix` | **yes** — **200** |
| 7 | If auth/URL failure: body documents failure (G2 scope) or **200** test payload | **yes** — **200** `Spot balance read OK` |

### H — Grafana SQL (H1 = F1)

| # | Check | Pass |
|---|-------|------|
| 8 | `POST /analytics/grafana/api/ds/query` (provisioned panel) **200** | **yes** (~95ms) |

### Regression

| # | Check | Pass |
|---|-------|------|
| 9 | OIDC-enabled deploy regression (if applicable) | **deferred** |
| 10 | Bundled-firefly deploy regression (if applicable) | **pass** (static — external profile, `database_host: postgres`) |

## Results summary

- **Automated (local):** cargo 103/103, vitest 2/2, build PASS; F2/G1 static contracts PASS.
- **Production:** PASS — F1 complete; rows F/G/H evidenced on `https://financegnome.omniflow.cc`.
- **Next:** `/release` in fresh subagent.
- **Evidence:** `sprints/quick/Q0009/verify-work-findings.md`, `uat.json`
