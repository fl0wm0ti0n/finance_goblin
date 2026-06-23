# Operator V1 runbook — Q0035 / BUG-0027

**Orchestrator run:** `auto-20260622-bug0027`
**Work item:** BUG-0027 — Firefly sync fails with 401 Unauthorized (PAT invalid/expired)
**Owner:** operator (manual — requires Firefly UI access + deploy host)
**Estimated duration:** ~15 min + 3h monitoring (3 scheduled syncs at hourly cadence)
**Pre-requisite:** `/verify-work` complete (CC PASS, CB/CD PENDING_OPERATOR)

## Purpose

Execute the operator-side acceptance verification for BUG-0027:
- **CB:** After regenerating Firefly PAT and updating `.env` + recreating container, sync succeeds; no `401` in `error_message`
- **CD:** ≥3 scheduled syncs succeed post-PAT regen (no silent 401 recurrence)
- Plus: deploy `0.22.1-bug0027` with the new `FireflyError::Unauthorized` taxonomy so the new error message is live

## Step 0: Pre-flight sanity (2 min)

Confirm deploy host + source are in expected state:

```bash
ls -la /workdir/financegoblin/deploy.sh
ls -la /workdir/financegoblin/.env
ls -la /workdir/dev_git/finance_goblin/backend/src/firefly/mod.rs
```

Expected: all three exist. `deploy.sh` is executable; `.env` has `FIREFLY_PERSONAL_ACCESS_TOKEN` (current stale/expired value); `mod.rs` contains the new `Unauthorized` variant (grep for `FireflyError::Unauthorized`).

## Step 1: Deploy `0.22.1-bug0027` (5 min)

```bash
RELEASE_TAG=0.22.1-bug0027 bash /workdir/financegoblin/deploy.sh
```

Or equivalently via wrapper:

```bash
RELEASE_TAG=0.22.1-bug0027 bash /workdir/dev_git/finance_goblin/scripts/deploy-omniflow.sh
```

**Expected output:**
```
Deploy: project=financegoblin release=0.22.1-bug0027 build_id=<short sha>
...docker build / up -d...
Health (traefik network — from container):
{"status":"ok"}
{"build_id":"...","release_tag":"0.22.1-bug0027","build_timestamp":"..."}
Done. Public URL: https://financegnome.omniflow.cc
```

**Verify after deploy:**

```bash
# Health endpoint
docker compose --project-name financegoblin \
  --env-file /workdir/financegoblin/.env \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external \
  exec -T flow-finance-ai curl -sf http://localhost:8080/health

# Build metadata (must show new release_tag)
docker compose --project-name financegoblin \
  --env-file /workdir/financegoblin/.env \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external \
  exec -T flow-finance-ai curl -sf http://localhost:8080/api/v1/meta/build-info
```

**Expected:** `release_tag: "0.22.1-bug0027"`. If not, rebuild did not pick up the new tag — re-run deploy.

## Step 2: Regenerate PAT in Firefly UI (3 min)

This step MUST be done in a web browser against the Firefly III instance.

1. Open Firefly III profile URL (typically `${FIREFLY_BASE_URL}/profile` — check `.env` for `FIREFLY_BASE_URL`)
2. Log in as the API user
3. Navigate to **Profile → OAuth → Personal Access Tokens** (or **Profile → API tokens**)
4. Click **Create new token** (or **Regenerate** if available)
5. Name: `finance-goblin-<date>` (e.g. `finance-goblin-20260622`)
6. **Copy the generated token immediately** — it is shown ONCE
7. Store safely (clipboard / password manager) for Step 3

**Warning:** Regenerating a PAT invalidates the previous one. If any OTHER system reads `FIREFLY_PERSONAL_ACCESS_TOKEN` from the same `.env`, it will break too. Check with `grep -r FIREFLY_PERSONAL_ACCESS_TOKEN /workdir` before regenerating.

## Step 3: Update `.env` with new PAT (2 min)

```bash
# Backup old .env
cp /workdir/financegoblin/.env /workdir/financegoblin/.env.bak-$(date -u +%Y%m%d-%H%M%S)

# Edit FIREFLY_PERSONAL_ACCESS_TOKEN
nano /workdir/financegoblin/.env
# Replace the value of FIREFLY_PERSONAL_ACCESS_TOKEN with the new token (980 chars)
# Ensure no trailing newline (some editors auto-add one)
```

**Verify:**

```bash
# Character count (should match Firefly's stated length, e.g. ~980)
grep -E '^FIREFLY_PERSONAL_ACCESS_TOKEN=' /workdir/financegoblin/.env | cut -d= -f2- | tr -d '"' | wc -c

# No trailing newline check
grep -E '^FIREFLY_PERSONAL_ACCESS_TOKEN=' /workdir/financegoblin/.env | cut -d= -f2- | tr -d '"' | od -c | tail -1
# Last byte should NOT be `\n`
```

## Step 4: Recreate container (2 min)

```bash
cd /workdir/financegoblin && \
AUTHENTIK_SECRET_KEY=unused-external-profile \
docker compose \
  --project-name financegoblin \
  --env-file .env \
  -f docker-compose.yml \
  -f docker-compose.external.yml \
  -f docker-compose.build.yml \
  --profile external \
  up -d --force-recreate flow-finance-ai
```

Note: `AUTHENTIK_SECRET_KEY` is not used in external profile but must be set to avoid compose warning / error.

**Verify container is up:**

```bash
docker compose --project-name financegoblin --profile external ps
# Container `financegoblin-flow-finance-ai-1` should be `Up (healthy)`
```

## Step 5: Trigger manual sync + verify status (3 min)

```bash
# Trigger manual sync
docker exec financegoblin-flow-finance-ai-1 \
  curl -sf -X POST http://localhost:8080/api/v1/sync/trigger \
    -H 'Content-Type: application/json' \
    -d '{"trigger":"manual"}'

# Wait ~30 seconds for sync to complete, then:
docker exec financegoblin-flow-finance-ai-1 \
  curl -sf http://localhost:8080/api/v1/sync/status | jq .
```

**Expected (CB PASS):**
```json
{
  "state": "completed",
  "started_at": "...",
  "completed_at": "...",
  "error_message": null,
  "last_firefly_run": "...",
  "entity_counts": {
    "transactions": <positive>,
    "accounts": <positive>,
    ...
  }
}
```

**If state=failed:**
- `error_message` contains `"firefly_personal_access_token invalid or expired"` → **Step 2 PAT regen error** — regenerate again
- `error_message` contains `"firefly_personal_access_token_missing"` → **Step 3 .env error** — re-check .env, ensure not empty
- `error_message` contains `"unexpected status 401"` → **BUG-0027 NOT fixed** — rebuild did not pick up new variant; open new bug
- Any other → investigate sync logs (`docker logs financegoblin-flow-finance-ai-1`)

**CB verdict:** PASS iff `state: completed` + `error_message: null` + entity counts > 0.

## Step 6: Monitor ≥3 scheduled syncs (hourly, 3h minimum)

Scheduled syncs run hourly by default (check `SYNC_CRON` in `.env`, typically `0 * * * *`).

Option A: **Wait 3 hours and check sync_runs history:**

```bash
docker exec financegoblin-flow-finance-ai-1 \
  psql -U finance -d finance_goblin -c \
  "SELECT started_at, state, error_message, trigger \
   FROM sync_runs \
   WHERE started_at > NOW() - INTERVAL '4 hours' \
   ORDER BY started_at DESC \
   LIMIT 5;"
```

Option B: **Watch logs in real time:**

```bash
docker logs -f financegoblin-flow-finance-ai-1 2>&1 | grep -E "(sync_run|firefly|401|Unauthorized)"
```

**Expected (CD PASS):**
- At least 3 `state=completed` sync_runs with `error_message IS NULL`
- No `401` / `Unauthorized` entries in logs
- `trigger` column shows `scheduled` (proves cron is firing)

**CD verdict:** PASS iff ≥3 scheduled sync_runs all `completed` + no 401 in logs.

## Step 7: Finalize operator V1

**If CB + CD both PASS:**
1. Update `sprints/quick/Q0035/progress.md`: mark V1 → DONE
2. Update `sprints/quick/Q0035/uat.json`: set CB/CD results to `pass`
3. Append state.md checkpoint (operator V1 complete; BUG-0027 fully closed)
4. Move to `/release` in fresh subagent (release agent can now mark BUG-0027 DONE/released)

**If CB FAIL (401 persists):**
1. Open new bug BUG-0028 (re-occurrence of 401; likely Firefly API endpoint change or Firefly version regression)
2. Check docker logs for `FireflyError::Unauthorized` Display message — confirms new code is live
3. If Display message NOT in logs → deploy did not pick up new variant; re-check Step 1 build_id
4. Do NOT close BUG-0027

**If CD FAIL (only some syncs succeed):**
1. Inspect failing sync_run `error_message` column
2. If 401 → PAT may have been rate-limited or secondary auth issue → open new bug
3. If non-401 error → orthogonal issue (network timeout, Firefly restart); BUG-0027 remains closed

## Step 8: Smoke the public URL (2 min, optional)

```bash
curl -sf https://financegnome.omniflow.cc/health
curl -sf https://financegnome.omniflow.cc/api/v1/meta/build-info | jq .
```

Build metadata should show `release_tag: "0.22.1-bug0027"`. Public URL sanity check only — the operator V1 work is local to the host.

## Rollback (if anything goes wrong)

See `sprints/quick/Q0035/release-plan.md` § Rollback strategy.

## Acceptance → V1 mapping

| Acceptance row | V1 step | Verdict source |
|---------------|---------|----------------|
| **CB** | Steps 2 → 3 → 4 → 5 | Step 5 output: `state: completed` + `error_message: null` |
| **CC** | Step 1 (deploy) | Code verified in `/verify-work` — Display message frozen; new variant live after deploy |
| **CD** | Step 6 | ≥3 sync_runs in `sync_runs` table with `state=completed, trigger=scheduled` |

## Isolation evidence

- `phase_id`: verify-work
- `role`: qa (runbook authored by QA; executed by operator in release phase)
- `fresh_context_marker`: verify-work-20260622-bug0027-qa-fresh
- `timestamp`: 2026-06-22T22:58:00Z
- `evidence_ref`: sprints/quick/Q0035/operator-v1-runbook.md
