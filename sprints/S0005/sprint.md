# Sprint S0005

**ID:** S0005  
**Story:** US-0005 — Wealth analysis, budget drift & scarcity alerts  
**Status:** PLANNED  
**Created:** 2026-05-31

## Goal

Deliver wealth analysis and the unified Alert Engine on top of US-0001 mirror accounts, US-0002 forecast snapshots, US-0003 subscription alert boundary, and US-0004 active plan: migration 005 persistence, WealthService net worth + daily snapshots, AlertService (scarcity / budget drift / plan viability), post-sync `"alerts"` phase, wealth/alert REST API, React `/wealth` + `/alerts` + header bell, Grafana Dashboard 4 partial + Dashboard 1 threshold centralization, tests, and operator user guide.

## Scope

- Migration `005_alerts_wealth.sql` — `alert_config`, `alerts`, `net_worth_snapshots` (DEC-0027)
- Config: TOML `[alerts]` + startup UPSERT to `alert_config` singleton (DEC-0029)
- WealthService: asset account breakdown, mixed-currency flag, daily snapshot upsert (DEC-0025)
- Alert Engine: scarcity (household), budget drift (plan category targets), plan viability (DEC-0026)
- Alert lifecycle: fingerprint dedup, acknowledge/dismiss/resolve (DEC-0027)
- Sync pipeline: inline `"alerts"` phase after forecast+plan hook; failure non-blocking (DEC-0028)
- REST API: wealth breakdown/history + alert inbox/unread/acknowledge/dismiss
- React: enable Wealth nav; `/wealth` overview; header bell + Popover; `/alerts` inbox (DEC-0030)
- Grafana: `portfolio.json` (uid `portfolio`) + Dashboard 1 `$scarcity_threshold` variable (DEC-0029, DEC-0030)
- Tests and operator user guide (`docs/user-guides/US-0005.md`)

**Out of scope:** Full crypto PnL (US-0007), Grafana Alertmanager rules, AI tool wiring (US-0006), subscription alert migration to unified inbox, multi-currency FX conversion, per-account scarcity alerts.

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Mixed-currency headline misleading | `mixed_currency` flag + mandatory UI banner | R-0021, DEC-0025 |
| Budget drift MTD proration skew | Category-targeted adjustments only; document one-time delta limitation | R-0022, DEC-0026 |
| Plan viability on stale baseline | Bind computation IDs in alert `context`; stale metadata | R-0022, R-0019 |
| Mutex duration growth | Log alerts phase timing; defer queue if combined > ~30s | R-0024, DEC-0028 |
| US-0003 vs unified inbox confusion | Cross-link only; document separate surfaces | R-0023, DEC-0030 |
| Dashboard 1 variable coercion | Test `$scarcity_threshold::numeric` in SQL | R-0025, DEC-0029 |
| Snapshot gaps on failed sync | Show last snapshot date; flat line acceptable in Grafana | R-0021 |
| Acknowledged-but-active UX | Copy: "Acknowledged — condition still active" | R-0023 |

## Definition of Done

- All 12 sprint tasks complete (`T-0049` … `T-0060`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` US-0005
- Post-sync pipeline runs snapshot + alert evaluation in `"alerts"` phase
- `/wealth` and `/alerts` reachable; header bell shows unread count
- Grafana Dashboard 4 loads with uid `portfolio`; Dashboard 1 scarcity line reads from `alert_config`
- User guide published at `docs/user-guides/US-0005.md`
- No Firefly write operations introduced

## Architecture references

- `docs/engineering/architecture.md` — US-0005
- Decisions: DEC-0025 … DEC-0030
- Research: R-0021 … R-0026
- Depends on: US-0001 mirror accounts; US-0002 forecast; US-0003 subscription alerts boundary; US-0004 active plan + category adjustments
