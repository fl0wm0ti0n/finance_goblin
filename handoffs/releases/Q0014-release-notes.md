# Quick Release Notes — Q0014+Q0015 / BUG-0012

**Quick tasks:** Q0014 (DEC-0067), Q0015 (follow-up)  
**Bug:** BUG-0012 — Forecast monthly Income/Fixed buckets always zero  
**Date:** 2026-06-06  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0012 rows AG/AH)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --lib` (142/142 @ 2026-06-06 release)
2. **QA completion gate:** PASS — `sprints/quick/Q0014/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0014/uat.json`, `sprints/quick/Q0014/uat.md`, `handoffs/verify_work_to_release.md`; omniflow rows AG/AH live curl PASS (browser OIDC deferred)
4. **Isolation compliance gate:** PASS — discovery through verify-work checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — verify-work tuple `runtime-proof-verify-work-20260605-bug0012-q0014-002`
6. **Release finalization gate:** PASS

---

## Summary

Fixes monthly forecast bucket decomposition on US-0010 external profile (`financegnome.omniflow.cc`):

| Scope | Fix |
|-------|-----|
| **Q0014 / DEC-0067** | Component-level `monthly_map` attribution; `RecurringPattern.category_id` carry; retire net-delta `categorize_delta` |
| **Q0015 follow-up** | Payee-key due matching; household income from revenue accounts; standing-order forecast scope; German `category_buckets`; payroll payee collapse |

**Production proof (account 114):** Fixed **2073.85** (Jun 2026); Income **3266.16** (Jul 2026+).

---

## Run

**Target service (external profile):** `flow-finance-ai` — backend rebuild/recreate; operator **Manual Full Firefly sync** required for AG/AH acceptance.

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator prerequisite (V1 — required for AG/AH):**

1. Deploy Q0014+Q0015 backend image to omniflow.
2. Ensure `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` configured (names only).
3. Trigger **Manual Full Firefly sync** + forecast recompute.
4. Recreate if needed:

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

- `start_command`: docker compose command above
- `runtime_mode`: remote (omniflow external US-0010); `AUTH_DEV_BYPASS=true` when OIDC unset
- `runtime_context_ref`: `docs/engineering/runbook.md` (§16 BUG-0012 hotfix)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- `service_port`: 443 (HTTPS via Traefik)
- `health_endpoint`: `GET /health`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Full Firefly sync gate |
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite §11) |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(AG)** | `GET /api/v1/forecast/monthly?account_id=114` | `income > 0` in forecast horizon (Jul+ when salary projects from revenue acct) |
| **(AH)** | `GET /api/v1/forecast/monthly?account_id=114` | `fixed_costs > 0` (e.g. Jun **2073.85**) |
| Meta | `GET /api/v1/forecast/meta` | `computed_at` post-deploy (e.g. 2026-06-05T21:39:42Z) |
| Stack | `GET /health` | HTTP 200 |
| Sync | `GET /api/v1/sync/status` | Last manual success run recorded |

**Automated (release):**

```bash
cd backend && cargo test --lib
```

**Live (verify-work):** account 114 — Jun `fixed_costs: 2073.85`, Jul `income: 3266.16`, `fixed_costs: 652.09`.

**Conditional:** If AG/AH still zero after sync, extend `[forecast.category_buckets]` per runbook §16 TOML checklist.

---

## Credentials

- `FIREFLY_PERSONAL_ACCESS_TOKEN` — operator `.env` / secret store (never inline)
- Traefik basic auth — operator shell / password manager
- No inline secrets in release artifacts

---

## Changes

| Area | Summary |
|------|---------|
| `forecast/project.rs` | Per-component monthly_map (DEC-0067); household income merge; subscription category inherit |
| `forecast/recurring.rs` | `category_id` mode carry; payee-key due; 6-tx amount stability |
| `forecast/categories.rs` | `resolve_bucket`, `accumulate_bucket` |
| `forecast/types.rs` | `RecurringPattern.category_id` |
| Config | German `category_buckets`; standing-order patterns |
| Runbook | §16 BUG-0012 hotfix operator checklist |

**Linked decisions:** DEC-0067  
**Deferred epic:** US-0015 (AI-assisted bucket mapping)

---

## Known Issues

- Jun income 0 acceptable when salary due projects from July (last mirror pay before horizon) — operator accepted recurring semantics
- Browser Monthly tab / OIDC regression smoke deferred (API PASS)
- US-0015 AI bucket enrichment out of scope

---

## Regression scope

- Daily balance / milestones / horizons unchanged
- Rolling residual → Variable only; unmapped → Variable
- DEC-0007 baseline math preserved; no frontend changes required
