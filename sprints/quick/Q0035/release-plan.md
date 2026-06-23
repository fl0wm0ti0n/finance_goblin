# Release plan — Q0035 / BUG-0027

**Orchestrator run:** `auto-20260622-bug0027`
**Work item:** BUG-0027 — Firefly sync fails with 401 Unauthorized
**Release version:** `0.22.1-bug0027` (patch bump — bugfix-only, no US)
**Release target:** `omniflow-external` (`docs/engineering/release-targets.json[0]`)
**Public URL:** https://financegnome.omniflow.cc
**Build date:** 2026-06-22 (verify-work prep; actual build at operator discretion)

## Release scope (blast radius)

| File | Change |
|------|--------|
| `backend/src/firefly/mod.rs` L37-40 | Added `FireflyError::Unauthorized` variant + frozen Display message |
| `backend/src/firefly/mod.rs` L156-158 | `status == StatusCode::UNAUTHORIZED` → `Err(FireflyError::Unauthorized)` before retry + `UnexpectedStatus` fallthrough |
| `backend/tests/firefly_integration.rs` L155-192 | Added `test_firefly_401_returns_unauthorized_variant` (wiremock 401 mock → assert variant + error message) |

**Layer:** backend only. No frontend change, no migration, no runbook change, no DEC.
**Blast radius:** Firefly HTTP request path ONLY. `sync/mod.rs` inherits new variant via `e.to_string()` (no source change). Zero risk to: forecast, wealth, exchange, Grafana, OAuth, meta.

## Frozen gates from architecture § BUG-0027

| Gate | State |
|------|-------|
| GATE-ERROR-1 | ✅ frozen (unit variant, no fields) |
| GATE-MESSAGE-1 | ✅ frozen (Display string) |
| GATE-302-HANDLING | ✅ closed (Accept: application/json → 401) |
| GATE-PREFLIGHT-1 | ❌ deferred (future US) |
| GATE-TEST-1 | ✅ frozen (wiremock 401 → Unauthorized) |
| GATE-DEC-1 | ✅ closed (no new DEC) |

## Verify-work gates (verify-work-20260622-bug0027-qa-fresh)

| Gate | Result | Command |
|------|--------|---------|
| CC code verification | ✅ PASS | Read `backend/src/firefly/mod.rs` L37-40 — Display string exact-match spec |
| 401 arm precedes UnexpectedStatus | ✅ PASS | Read `backend/src/firefly/mod.rs` L156-166 — Unauthorized check (L156) before retry (L160) + UnexpectedStatus (L166) |
| Integration test asserts Unauthorized | ✅ PASS | Read `backend/tests/firefly_integration.rs` L155-192 |
| `cargo test --test firefly_integration` | ✅ **2/2 PASS** | `test_firefly_401_returns_unauthorized_variant` + `sync_issues_only_get_requests_to_firefly` |

## Build command

Primary (via wrapper):

```bash
bash /workdir/financegoblin/deploy.sh
```

Or equivalently:

```bash
bash /workdir/dev_git/finance_goblin/scripts/deploy-omniflow.sh
```

Both wrappers invoke the canonical `/workdir/financegoblin/deploy.sh`.

## Build environment

| Variable | Source |
|----------|--------|
| `RELEASE_TAG` | `0.22.1-bug0027` (override default `0.22.0-us0022`) |
| `BUILD_ID` | `$(git -C /workdir/dev_git/finance_goblin rev-parse --short HEAD)` |
| `BUILD_TIMESTAMP` | `$(date -u +%Y-%m-%dT%H:%M:%SZ)` |
| `AUTHENTIK_SECRET_KEY` | `unused-external-profile` |
| `ENV_FILE` | `/workdir/financegoblin/.env` |

**Recommended invocation:**

```bash
RELEASE_TAG=0.22.1-bug0027 bash /workdir/financegoblin/deploy.sh
```

The wrapper:
- Runs `docker compose build --profile external flow-finance-ai` with the three compose files (`docker-compose.yml`, `docker-compose.external.yml`, `docker-compose.build.yml`)
- Runs `docker compose up -d --force-recreate --profile external flow-finance-ai grafana`
- Hits `/health` + `/api/v1/meta/build-info` as post-deploy sanity

## Deploy target composition

- Compose project: `financegoblin`
- Services rebuilt: `flow-finance-ai`, `grafana`
- Network: `traefik` (TLS via Traefik `websecure` entrypoint, router `financegnome`)
- Deploy root: `/workdir/financegoblin`
- Source root: `/workdir/dev_git/finance_goblin`

## Rollback strategy

If post-deploy smoke fails:

```bash
# Option A — revert to previous image if tagged
docker compose --project-name financegoblin \
  --env-file /workdir/financegoblin/.env \
  -f /workdir/financegoblin/docker-compose.yml \
  -f /workdir/financegoblin/docker-compose.external.yml \
  -f /workdir/financegoblin/docker-compose.build.yml \
  --profile external \
  down

# Then re-up with previous image tag (if retained in local registry)
# or git revert the source commit + rebuild.

# Option B — source revert
cd /workdir/dev_git/finance_goblin && git checkout HEAD~1
RELEASE_TAG=0.22.0-us0022 bash /workdir/financegoblin/deploy.sh
```

**Rollback scope:** Since this bugfix touches only `backend/src/firefly/mod.rs` + test, rollback cannot destabilize unrelated layers. Pre-deploy state (2026-06-16 deploy of `0.22.0-us0022`) remains the stable baseline.

## Post-deploy verification (operator V1)

See `sprints/quick/Q0035/operator-v1-runbook.md` for full step list.

**Summary:** After deploy, operator MUST:
1. Regenerate PAT in Firefly profile → API tokens
2. Update `FIREFLY_PERSONAL_ACCESS_TOKEN` in `/workdir/financegoblin/.env`
3. Recreate container (`docker compose up -d --force-recreate flow-finance-ai`)
4. Verify `GET /api/v1/sync/status` → `state: completed`
5. Monitor ≥3 scheduled syncs (hourly cadence) — confirm no 401 recurrence

## Release-pass criteria (for next phase: `/release`)

| Criteria | Status after verify-work |
|----------|--------------------------|
| `cargo test --lib` | deferred to release phase (previously 221/221 baseline) |
| `cargo test --test firefly_integration` | ✅ 2/2 PASS |
| `cargo test sync --lib` | ✅ 24/24 PASS (in qa phase) |
| `cargo test sync --test bug0025_sync_transaction_window` | ✅ 3/3 PASS |
| CC acceptance | ✅ PASS (code verified independently) |
| CB acceptance | ⏸ PENDING_OPERATOR (V1 runbook ready) |
| CD acceptance | ⏸ PENDING_OPERATOR (V1 runbook ready) |
| Operator V1 smoke | ⏸ PENDING (blocked on PAT regen + deploy) |

## Release artifacts for `/release` phase

- `handoffs/releases/Q0035-release-notes.md`
- `sprints/quick/Q0035/release-findings.md`
- Traceability index update: BUG-0027 row → DONE / released + evidence refs

## Isolation evidence (verify-work)

- `phase_id`: verify-work
- `role`: qa
- `fresh_context_marker`: verify-work-20260622-bug0027-qa-fresh
- `timestamp`: 2026-06-22T22:58:00Z
- `evidence_ref`: sprints/quick/Q0035/release-plan.md, sprints/quick/Q0035/operator-v1-runbook.md, sprints/quick/Q0035/uat.json, sprints/quick/Q0035/uat.md
