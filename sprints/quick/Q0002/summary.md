# Q0002 — Browser errors on financegnome (2026-06-03)

## What the user saw

| Console message | Cause |
|-----------------|-------|
| `contentscript.js` MaxListenersExceededWarning, ObjectMultiplex | **Browser extension** (e.g. MetaMask/wallet) — not Flow Finance AI |
| `/favicon.ico` 404 | No favicon asset (fixed: `frontend/public/favicon.svg`) |
| `(index)` 404 | **App not serving HTTP** — container crash-loop on DB bootstrap |

## Root cause (runtime)

`flow-finance-ai` logs:

```
password authentication failed for user "flow"
bootstrap_reason=database_bootstrap_failed_connect
```

`.env` uses Postgres user `flow` (or wrong password). Host omniflow Postgres must match `DATABASE_USER` / `DATABASE_PASSWORD`. If the role cannot `CREATEDB`, set `DATABASE_BOOTSTRAP_URL` to an admin URL (see `.env.example`).

Until DB auth succeeds, Traefik has no healthy backend → **404 on `/`**.

## Code fix

Bootstrap failures were wrapped as `ConfigError::MissingEnv`, producing the confusing message:

`missing required environment variable: database_bootstrap_failed_connect`

Now the fatal line is the real bootstrap error, e.g.:

`database_bootstrap_failed_connect: maintenance database unreachable (...)`

## Operator fix (required)

1. Edit `.env`: align `DATABASE_USER` / `DATABASE_PASSWORD` with host Postgres (`.env.example` uses `finance`; your logs show `flow`).
2. Optional: `DATABASE_BOOTSTRAP_URL=postgres://...@host.docker.internal:5432/postgres` for admin create.
3. Restart:

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build
```

4. Confirm: `docker logs finance_goblin-flow-finance-ai-1 2>&1 | tail -5` shows migrations / listening, not bootstrap retry.

## Rebuild note

Favicon + error-message fix need image rebuild to take effect in Docker.
