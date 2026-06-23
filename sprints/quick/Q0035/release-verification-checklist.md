# Release Verification Checklist — Q0035 (BUG-0027)

**Sprint:** Q0035  
**Bug:** BUG-0027 — Firefly sync fails with 401 Unauthorized (PAT invalid/expired after deploy)  
**Release version:** `0.22.1-bug0027`  
**Checklist purpose:** document operator actions required to complete CB/CD verification post-deploy  
**Date created:** 2026-06-22

---

## Pre-deploy checks

- [ ] Confirm `/workdir/dev_git/finance_goblin/backend/src/firefly/mod.rs` contains `FireflyError::Unauthorized` variant:
  ```bash
  rg 'FireflyError::Unauthorized' /workdir/dev_git/finance_goblin/backend/src/firefly/mod.rs
  ```
- [ ] Confirm deploy script exists and is executable:
  ```bash
  ls -la /workdir/financegoblin/deploy.sh
  ```
- [ ] Note current `FIREFLY_PERSONAL_ACCESS_TOKEN` length (should be ~980 chars):
  ```bash
  grep -E '^FIREFLY_PERSONAL_ACCESS_TOKEN=' /workdir/financegoblin/.env | cut -d= -f2- | wc -c
  ```
- [ ] Backup current `.env`:
  ```bash
  cp /workdir/financegoblin/.env /workdir/financegoblin/.env.bak-pre-bug0027
  ```

---

## Step 1: Deploy version `0.22.1-bug0027`

- [ ] Run deploy command:
  ```bash
  RELEASE_TAG=0.22.1-bug0027 bash /workdir/financegoblin/deploy.sh
  ```
- [ ] Verify health endpoint:
  ```bash
  docker exec financegoblin-flow-finance-ai-1 curl -sf http://localhost:8080/health
  ```
  **Expected:** `{"status":"ok"}`
- [ ] Verify build metadata — `release_tag` MUST be `0.22.1-bug0027`:
  ```bash
  docker exec financegoblin-flow-finance-ai-1 curl -sf http://localhost:8080/api/v1/meta/build-info | jq .
  ```
- [ ] If `release_tag` ≠ `0.22.1-bug0027`: re-run deploy; do NOT proceed to PAT regen

---

## Step 2: Regenerate Firefly PAT

- [ ] Open Firefly III profile URL (check `FIREFLY_BASE_URL` in `.env` → append `/profile`)
- [ ] Log in as the API user
- [ ] Navigate to **Profile → OAuth → Personal Access Tokens** (or **Profile → API tokens**)
- [ ] Create new token (name: `finance-goblin-20260622` or similar)
- [ ] **Copy generated token immediately** — shown ONLY ONCE
- [ ] Store in clipboard / password manager
- [ ] **Warning:** Old token is invalidated; no other reader should depend on the same PAT

---

## Step 3: Update `.env` with new PAT

- [ ] Edit `/workdir/financegoblin/.env` — replace value of `FIREFLY_PERSONAL_ACCESS_TOKEN`
- [ ] Ensure no trailing newline (some editors auto-add):
  ```bash
  grep -E '^FIREFLY_PERSONAL_ACCESS_TOKEN=' /workdir/financegoblin/.env | cut -d= -f2- | tr -d '"' | od -c | tail -1
  ```
- [ ] Verify character count matches Firefly's stated length:
  ```bash
  grep -E '^FIREFLY_PERSONAL_ACCESS_TOKEN=' /workdir/financegoblin/.env | cut -d= -f2- | tr -d '"' | wc -c
  ```

---

## Step 4: Recreate container

- [ ] Force recreate `flow-finance-ai`:
  ```bash
  cd /workdir/financegoblin && \
  AUTHENTIK_SECRET_KEY=dummy \
  docker compose \
    --project-name financegoblin \
    --env-file .env \
    -f docker-compose.yml \
    -f docker-compose.external.yml \
    --profile external \
    up -d --force-recreate flow-finance-ai
  ```
- [ ] Verify container is `Up (healthy)`:
  ```bash
  docker compose --project-name financegoblin --profile external ps
  ```

---

## Step 5: Verify sync status (CB acceptance)

- [ ] Trigger manual sync:
  ```bash
  docker exec financegoblin-flow-finance-ai-1 curl -sf -X POST http://localhost:8080/api/v1/sync/trigger -H 'Content-Type: application/json' -d '{"trigger":"manual"}'
  ```
- [ ] Wait ~30s for sync to complete
- [ ] Check sync status:
  ```bash
  docker exec financegoblin-flow-finance-ai-1 curl -sf http://localhost:8080/api/v1/sync/status | jq .
  ```
- [ ] **Expected (CB PASS):** `"state": "completed"`, `"error_message": null`, `entity_counts` > 0
- [ ] If `state=failed`:
  - `error_message` contains `"firefly_personal_access_token invalid or expired"` → Step 2 PAT regenerate failed; regenerate again
  - `error_message` contains `"unexpected status 401"` → BUG-0027 code NOT fixed in this deploy; re-check Step 1 `release_tag`
- [ ] **Record CB verdict:** PASS / FAIL

---

## Step 6: Monitor ≥3 scheduled syncs (CD acceptance)

- [ ] Wait 3h+ for scheduled syncs (hourly cron: `SYNC_CRON` in `.env`, typically `0 * * * *`)
- [ ] Check sync_runs history:
  ```bash
  docker exec financegoblin-flow-finance-ai-1 psql -U finance -d finance_goblin -c \
    "SELECT started_at, state, error_message, trigger \
     FROM sync_runs \
     WHERE started_at > NOW() - INTERVAL '4 hours' \
     ORDER BY started_at DESC LIMIT 5;"
  ```
- [ ] **Expected (CD PASS):** ≥3 rows with `state=completed`, `error_message IS NULL`, `trigger=scheduled`
- [ ] Verify no 401 in logs:
  ```bash
  docker logs financegoblin-flow-finance-ai-1 2>&1 | grep -E '(401|Unauthorized)'
  ```
  **Expected:** zero matches
- [ ] **Record CD verdict:** PASS / FAIL

---

## Step 7: Finalize acceptance

- [ ] If CB + CD PASS:
  - Append state.md checkpoint: operator V1 complete
  - Update `sprints/quick/Q0035/progress.md`: V1 → DONE
  - Update `sprints/quick/Q0035/uat.json`: CB/CD results → `pass`
  - Update `docs/product/acceptance.md`: BUG-0027 rows — CB ✅, CD ✅
  - Update `docs/product/backlog.md`: BUG-0027 status → `DONE`
  - Run `/refresh-context` in new subagent to record V1 closure

- [ ] If CB FAIL:
  - Open new bug BUG-0028 (401 re-occurrence — likely Firefly regression or deploy miss)
  - Do NOT close BUG-0027

- [ ] If CD FAIL (partial syncs):
  - Inspect `error_message` column on failing sync_runs
  - If 401 → PAT secondary issue; open new bug
  - If non-401 → orthogonal issue; BUG-0027 remains closed

---

## Rollback (if deploy fails or 401 persists with new PAT)

```bash
cd /workdir/dev_git/finance_goblin && git revert <commit-hash>
RELEASE_TAG=0.22.0-us0022 bash /workdir/financegoblin/deploy.sh
```

---

## Checklist status tracking

| Field | Value |
|-------|-------|
| Created | 2026-06-22 |
| Deployed (`0.22.1-bug0027`) | ⏳ pending |
| PAT regenerated | ⏳ pending |
| `.env` updated | ⏳ pending |
| Container recreated | ⏳ pending |
| CB (sync status) | ⏳ pending |
| CD (≥3 scheduled syncs) | ⏳ pending |
| Finalized | ⏳ pending |
| **Operator:** | _fill on execution_ |
| **CB verdict:** | _fill on execution_ |
| **CD verdict:** | _fill on execution_ |
| **Execution timestamp:** | _fill on execution_ |
