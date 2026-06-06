# Q0009 — BUG-0003 omniflow production API 500 / Bitunix / Grafana SQL

| Field | Value |
|-------|-------|
| **ID** | Q0009 |
| **Type** | `/quick` |
| **Status** | PLAN-VERIFY PASS |
| **Bug** | BUG-0003 |
| **Created** | 2026-06-05 |
| **Architecture** | `architecture-20260605-bug0003` (`docs/engineering/architecture.md` § BUG-0003) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260605-q0009-bug0003`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0003 rows **(F)**, **(G)**, **(H)** |
| **Task count** | 4 (3 required + G2 gated) |
| **Next phase** | `/execute` |

## Goal

Close BUG-0003 on US-0010 external omniflow: fix `DATABASE_HOST` misconfiguration cascade (**F**/**H**), complete Q0008 E1 parity in `ExchangeService::new` (**G**), optional R-0058 futures auth (**G2** gated). **No new DEC** — extends DEC-0056, R-0052, R-0058.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| F — DB host | F1, F2 | ops + docs |
| G — Bitunix registry | G1, G2 (gated) | backend |
| H — Grafana SQL | H1 (= F1 verify) | ops |

**Out of scope:** BUG-0002/Q0008; H2 UID dedupe (optional follow-up); Traefik/proxy changes.

## Task summary

| ID | Title | Est. | Depends | Acceptance hook |
|----|-------|------|---------|-----------------|
| F2 | External-profile DATABASE_HOST guard | 1h | — | **(F)** docs |
| G1 | `effective_enabled()` in `ExchangeService::new` | 2h | — | **(G)** code |
| F1 | Operator `DATABASE_HOST=postgres` + recreate | 1h | — | **(F)(H)** operator |
| G2 | R-0058 futures auth spike | 1.5h | G1, F1 + smoke gate | **(G)** conditional |

**Total estimate:** ~5.5h (dev ~3.5h + operator F1 ~1h + gated G2 ~1.5h).

## Deploy order

```text
(F2 ∥ G1) code PR  →  deploy image  →  F1 operator recreate  →  smoke F/G/H
                                              └→ G2 only if bitunix test auth-fails (not unknown exchange)
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **(F)** | F1, F2 | Settings `database_host: postgres`; GET APIs **200** &lt;2s |
| **(G)** | G1, G2? | `POST …/bitunix/test` not **400** unknown exchange |
| **(H)** | F1 | Grafana `ds/query` **200** |
| Regression | post-F1 | Acceptance footer |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
