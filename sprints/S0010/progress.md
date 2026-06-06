# Progress — Sprint S0010

**Story:** US-0010  
**Sprint status:** EXECUTE COMPLETE → QA  
**Tasks:** 10/10 complete

| Task | Status | Notes |
|------|--------|-------|
| T-0109 | done | `firefly-iii` → `[bundled-firefly, standard, full]`; header comments updated |
| T-0110 | done | Traefik labels `${TRAEFIK_HOST:-…}`, `${TRAEFIK_MIDDLEWARE:-auth}` |
| T-0111 | done | Grafana `ports: !reset []`, traefik network, optional router via `${GRAFANA_TRAEFIK_HOST:+true}` |
| T-0112 | done | `.env.example` omniflow block expanded |
| T-0113 | done | `scripts/compose-config-check.sh` + `tests/run-tests.sh` integration |
| T-0114 | done | Runbook §7 cross-ref to compose-config-check |
| T-0115 | done | `docs/user-guides/US-0001.md` bundled-firefly start commands |
| T-0116 | done | `docs/user-guides/US-0010.md` operator guide |
| T-0117 | done | External overlay merged config assertions in compose-config-check |
| T-0118 | partial | Local PASS: config checks, build, step 7; host steps 1–6, 8 PENDING — see smoke-evidence.md |

## Test results

- `bash tests/run-tests.sh` — **PASS** (backend unit + frontend build + compose-config-check)
- `scripts/compose-config-check.sh` — **PASS** (standalone)
- `docker compose … --profile external build flow-finance-ai` — **PASS**

## Smoke status (T-0118)

| Scope | Status |
|-------|--------|
| Config-only (CI) | PASS |
| Docker build | PASS |
| Step 7 (no firefly-iii) | PASS (local config) |
| Steps 1–6, 8 (omniflow host) | PENDING — operator post-merge |

## Security

- No `finance_goblin/.env` or host stack secrets read
- Placeholder env only for compose config validation

## Next

- `/qa` via `handoffs/dev_to_qa.md`
- Operator completes omniflow smoke checklist on Debian host
