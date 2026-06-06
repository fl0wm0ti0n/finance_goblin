# Design Concept — US-0011

## Summary

US-0011 delivers a **unified analytics shell** inside financegnome: six provisioned Grafana dashboards appear as in-app routes under an **Analytics** sidebar group, embedded via same-origin kiosk iframes. Product charts on Forecast, Wealth, Planning, and Subscriptions remain ECharts-primary; Grafana routes provide SQL operational views without a separate public Grafana site.

## Goals

- Sidebar **Analytics** section with six in-app routes matching DEC-0012 dashboard uids
- Same-origin Grafana proxy so omniflow users need only `financegnome.omniflow.cc` (DEC-0056 + DEC-0057)
- Wealth portfolio entry migrates from external Grafana tab to `/analytics/portfolio`
- Traefik `auth` + optional OIDC compatible with embedded views (AC-3)
- Document future-chart contract: React-first for new features; Grafana embed as exception

## Non-goals

- Deprecating Grafana container or porting SQL panels to ECharts in this story
- Public Grafana Traefik host as default
- Grafana auth-proxy integration with OIDC (deferred decision gate)
- Changing dashboard JSON or datasource definitions

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0057 | `/analytics/grafana/` reverse proxy | Internal Grafana unreachable from browser; R-0054 |
| DEC-0056 | Internal Grafana default | Single URL; no double auth |
| DEC-0012 | Stable dashboard uids | Route map and iframe paths |
| Embed pattern | `kiosk=tv` iframes | Hide Grafana chrome; discovery + R-0054 |
| Deprecate | `VITE_GRAFANA_URL` | External tab breaks unified UX |

**UX references:** shadcn shell, full-width iframe content area, optional secondary links from ECharts pages to SQL views.
