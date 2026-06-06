# Design Concept — US-0005

## Summary

US-0005 delivers **wealth analysis** and a unified **Alert Engine** for proactive financial warnings: net worth aggregation from Firefly asset accounts, scarcity/budget-drift/plan-viability alerts with acknowledge/dismiss lifecycle, threshold config centralization (TOML + Grafana), React `/wealth` and `/alerts` with header notification bell, and Grafana Dashboard 4 (Portfolio partial).

Builds on US-0002 forecast paths, US-0004 active plan category targets, and US-0003 subscription alert boundary (unchanged).

## Goals

- Net worth: sum Firefly asset accounts; EUR reporting default; mixed-currency warning; crypto placeholder until US-0007 (DEC-0025, R-0021)
- Daily `net_worth_snapshots` post-sync for trend chart + Dashboard 4 (DEC-0025)
- Alert Engine: scarcity (household), budget drift (plan category targets), plan viability (DEC-0026, R-0022)
- Migration 005: `alerts`, `alert_config`, `net_worth_snapshots` (DEC-0027)
- Post-sync inline `"alerts"` phase after forecast+plan hook (DEC-0028, R-0024)
- Threshold centralization: TOML `[alerts]` → DB mirror → Grafana `$scarcity_threshold` (DEC-0029, R-0025)
- React `/wealth` + `/alerts` + header bell; subscription alerts cross-link only (DEC-0030)
- Grafana Dashboard 4 (`uid: portfolio`) partial provision (DEC-0030, R-0026)

## Non-goals

- Full crypto PnL and exchange connectors (US-0007)
- Grafana Alertmanager rules
- AI tool implementation (`get_budget_status`, `get_portfolio`) — US-0006
- Subscription alert migration to unified inbox
- Multi-currency FX conversion
- Redis/async alert queue
- Any write to Firefly III

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0025 | Asset sum + daily snapshots | Stable trend series; mixed-currency banner (R-0021) |
| DEC-0026 | Household scarcity + plan category drift | Actionable without false positives (R-0022) |
| DEC-0027 | Fingerprint dedup + dismiss-until-clear | PostgreSQL sufficient; no permanent suppress (R-0023) |
| DEC-0028 | Inline alerts phase | Consistent with DEC-0010; needs fresh forecast/plan (R-0024) |
| DEC-0029 | TOML + alert_config + Grafana variable | Single source; supersedes DEC-0012 hardcode (R-0025) |
| DEC-0030 | Separate inbox + bell | US-0003 boundary preserved (R-0023, R-0026) |

**UX references:** Finanzguru Gesamtvermögen + proactive warnings + alert inbox; Firefly asset account vocabulary; shadcn bell/Popover/table; ECharts optional on wealth page; Grafana Dashboard 4 — see `docs/product/vision.md`.

**Spec-pack traceability:** `docs/engineering/spec-pack/US-0005-crs.md`, `docs/engineering/spec-pack/US-0005-technical-specification.md`
