# Omniflow production deploy (host path)

Canonical **deploy root** on the homelab host: **`/workdir/financegoblin`**.

| Host file | Repo template |
|-----------|---------------|
| `/workdir/financegoblin/.env` | copy from `.env.example` + operator secrets |
| `/workdir/financegoblin/deploy.sh` | `scripts/deploy-omniflow.sh` → exec host script |
| `/workdir/financegoblin/docker-compose.build.yml` | `deploy/omniflow/docker-compose.build.yml` |

**Source:** `/workdir/dev_git/finance_goblin` (git checkout — not duplicated under deploy root).

## Quick deploy

```bash
cd /workdir/financegoblin
RELEASE_TAG=0.22.0-us0022 ./deploy.sh
```

Or from repo:

```bash
bash scripts/deploy-omniflow.sh
```

See `docs/engineering/runbook.md` § **Omniflow external deploy (US-0010)**.
