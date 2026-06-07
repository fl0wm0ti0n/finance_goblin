# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 8
- First archived heading: `## research-20260608-us0013 — US-0013 production ML hardening technical research`
- Last archived heading: `## research-20260608-us0013 — US-0013 production ML hardening technical research`
- Verification tuple (mandatory):
  - archived_body_lines=55
  - retained_body_lines=471

---

## research-20260608-us0013 — US-0013 production ML hardening technical research

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-08  
**Story:** US-0013  
**Orchestrator run:** auto-20260608-us0013-001  
**Next phase:** `/architecture`

### Summary

Web research completed for US-0013 external-profile ML enablement. Extended **[R-0071](docs/engineering/research.md#r-0071--us-0013-production-ml-enablement-on-omniflow-external-profile)** resolving all five discovery open questions: overlay profile-merge pattern (no duplicate sidecar), traefik-only network co-attachment, runtime health SLO vs compose healthcheck, unchanged ≥12 monthly-point gate, and dual CI guard (compose assert + wiremock integration). Recommends **DEC-0076** area decision for external overlay compose + env contract. No host `.env` or secrets read.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **Profile union** | [R-0071 §discovery](docs/engineering/research.md#r-0071--us-0013-production-ml-enablement-on-omniflow-external-profile) | **Overlay additive profiles** on single `stats-forecast` service — base `[full]` + overlay `[external]` → one container; reject duplicate service blocks |
| **Network** | [R-0071 §discovery](docs/engineering/research.md#r-0071--us-0013-production-ml-enablement-on-omniflow-external-profile) | **Traefik-only** sidecar attachment matching backend; `STATS_FORECAST_URL=http://stats-forecast:8090`; optional `${STATS_FORECAST_PORT:-8091}` host remap |
| **Sidecar SLO** | [R-0071 §discovery](docs/engineering/research.md#r-0071--us-0013-production-ml-enablement-on-omniflow-external-profile), [R-0044](docs/engineering/research.md#r-0044--statsforecast-sidecar-vs-rust-augurs-execution-model) | Runtime `health_ok()` gate (60s timeout); compose healthcheck advisory; first-sync cold-start skip acceptable per DEC-0052 |
| **Min history** | [R-0071 §discovery](docs/engineering/research.md#r-0071--us-0013-production-ml-enablement-on-omniflow-external-profile), [R-0045](docs/engineering/research.md#r-0045--seasonal-model-selection-autoets-mstl-fallback) | Keep `min_monthly_points=12`; Full sync prerequisite; `insufficient_history` skip path unchanged |
| **CI scope** | [R-0071 §discovery](docs/engineering/research.md#r-0071--us-0013-production-ml-enablement-on-omniflow-external-profile), [R-0053 §7](docs/engineering/research.md#r-0053--us-0010-omniflow-external-deploy-timescaledb-profile-guard-traefik-env-grafana-oidc-ci) | **Both** — extend `compose-config-check.sh` (3-service external set + traefik assert) **and** retain `forecast_ml_integration` |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| Profile union on base vs overlay? | **Overlay merge** — additive `external` profile on existing `stats-forecast`; one container for `external`, `full`, or `full+external` |
| Traefik-only vs dual-network sidecar? | **Traefik-only** — backend external merge has no default network; sidecar must share `traefik` for DNS |
| Sidecar health SLO on omniflow? | **Runtime GET `/health`** before ML phase; 30s compose start_period; document cold-start retry |
| Production min-history gate? | **≥12 monthly points** default unchanged; Full sync prerequisite; skip metadata on short history |
| CI: compose assert vs wiremock? | **Both** — update external-only service list to include `stats-forecast`; keep wiremock integration test |

### Risks surfaced (carry to architecture)

1. **Cold-start race** — first post-deploy sync may record `sidecar_unavailable` until healthcheck green
2. **DEC-0056 guard unchanged** — `minimal+external` must still exclude `firefly-iii`; sidecar addition does not relax profile-combination rules
3. **Host memory** — sidecar RSS on shared omniflow host (R-0044); operator monitoring note in runbook
4. **CI drift** — compose-config-check must update atomically with overlay execute or AC-1 regresses

### Recommended DEC-0076 area decisions

| ID | Scope | Proposed decision |
|----|-------|-------------------|
| **DEC-0076** | External ML compose contract | Overlay merge adds `stats-forecast` `profiles: [external]`, `networks: [traefik]`, optional host port remap; passthrough `FORECAST_ML_ENABLED` / `STATS_FORECAST_URL` on `flow-finance-ai`; preserves DEC-0049 default-off |
| *(sub-element)* | Failure semantics | No change — DEC-0052 skip-on-failure + DEC-0066 `sidecar_disabled` metadata |
| *(sub-element)* | CI guard | External-only `config --services` = `flow-finance-ai`, `grafana`, `stats-forecast`; traefik network assert in `compose-config-check.sh` |

### Recommended next steps

1. `/architecture` — formalize **DEC-0076**; update `architecture.md` § US-0013 / US-0009 external path
2. `/sprint-plan` — materialize US-0013-S1..S4 tasks per R-0071 decomposition

---

