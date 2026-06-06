# CRS — US-0011

## Purpose

Provide financegnome users a **single web application** for all analytics, including existing Grafana SQL dashboards, without opening a separate Grafana site for day-to-day use.

## Scope

**In scope**

- Analytics navigation group and six `/analytics/{slug}` pages with embedded Grafana dashboards
- Backend reverse proxy `/analytics/grafana/` → `grafana:3000`
- Environment contract (`GRAFANA_UPSTREAM`, `VITE_GRAFANA_EMBED_BASE`)
- Wealth page in-app link to portfolio analytics
- Operator user guide for single-URL UX and future-chart guidelines

**Out of scope**

- Grafana container removal; full SQL→React migration
- US-0010 Traefik/compose changes beyond Grafana anonymous env vars
- Application feature changes to forecast/subscription algorithms

## Acceptance criteria ref

See `docs/product/acceptance.md#US-0011` — 7 criteria (sidebar, in-app embed, Traefik auth, ECharts regression, Wealth migration, future-chart doc, single URL).

## Dependencies

- US-0010 released (DEC-0056 external profile)
- US-0001–US-0009 Grafana provisioning (DEC-0012 uids)
