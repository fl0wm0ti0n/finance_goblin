# Q0003 — Still can't connect (404)

## Browser console

| Message | Verdict |
|---------|---------|
| `contentscript.js` / ObjectMultiplex | Browser extension — ignore |
| `GET / 404` | **Symptom** — backend not running (Traefik no healthy upstream) |

## Root cause chain

1. `.env` credentials — fixed by operator
2. Disk full — fixed by operator cleanup
3. **TimescaleDB missing** on omniflow `postgres:latest` → app exit `database_bootstrap_failed_timescaledb` → restart loop → **404**

## Fix applied (2026-06-03)

On host `postgres` container:

- Installed `timescaledb-2-postgresql-16` (Timescale apt repo)
- `timescaledb-tune --quiet --yes`
- `docker restart postgres`
- `CREATE EXTENSION IF NOT EXISTS timescaledb` on `flow_finance_ai`
- Restarted `finance_goblin-flow-finance-ai-1`

## Verified

- Logs: `database_bootstrap_extension_ok`, `listening 0.0.0.0:8080`
- Container: **healthy**
- Internal: `GET /health` → `{"status":"ok"}`
- Public `https://financegnome.omniflow.cc/health` → **401** (Traefik `auth` middleware — expected; use basic auth)

## User action

Open `https://financegnome.omniflow.cc/` with Traefik basic-auth credentials (same as other omniflow apps). Hard refresh if cache still shows 404.

## Hardening (recommended)

Change omniflow postgres service image to `timescale/timescaledb:latest-pg16` so TimescaleDB is not lost on container recreate.
