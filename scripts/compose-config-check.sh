#!/usr/bin/env bash
# US-0010 compose config regression (T-0113, T-0117) — no live docker up required.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export DATABASE_PASSWORD=ci
export FIREFLY_APP_KEY='base64:32RandomCharactersMinimumRequired=='
export FIREFLY_DB_PASSWORD=ci
export AUTHENTIK_SECRET_KEY=ci

compose() {
  docker compose "$@"
}

fail() {
  echo "compose-config-check FAIL: $*" >&2
  exit 1
}

echo "==> compose-config-check: external-only services (AC-1)"
services=$(compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external config --services | sort | tr '\n' ' ')
expected="flow-finance-ai grafana stats-forecast "
if [[ "$services" != "$expected" ]]; then
  fail "external profile services expected [flow-finance-ai grafana stats-forecast], got [$services]"
fi
echo "    OK: $services"

echo "==> compose-config-check: anti-combination minimal+external (AC-1)"
bad=$(compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile minimal --profile external config --services)
if echo "$bad" | grep -qE '^(firefly-iii|postgres)$'; then
  fail "minimal+external must not include firefly-iii or postgres; got: $(echo "$bad" | tr '\n' ' ')"
fi
echo "    OK: no firefly-iii or postgres in minimal+external merge"

echo "==> compose-config-check: greenfield minimal+bundled-firefly (AC-1 regression)"
good=$(compose --profile minimal --profile bundled-firefly config --services)
if ! echo "$good" | grep -qx 'firefly-iii'; then
  fail "minimal+bundled-firefly must include firefly-iii"
fi
echo "    OK: firefly-iii present in minimal+bundled-firefly"

echo "==> compose-config-check: minimal alone excludes firefly-iii"
minimal_only=$(compose --profile minimal config --services)
if echo "$minimal_only" | grep -qx 'firefly-iii'; then
  fail "minimal alone must not include firefly-iii"
fi
echo "    OK: minimal alone has flow-finance-ai + grafana only"

echo "==> compose-config-check: Traefik env defaults (AC-3)"
config=$(compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external config)
if echo "$config" | grep -q 'financegnome.omniflow.cc'; then
  echo "    OK: TRAEFIK_HOST default host present in merged config"
else
  fail "merged config missing financegnome.omniflow.cc (TRAEFIK_HOST default)"
fi
if echo "$config" | grep -q 'traefik.http.routers.financegnome.middlewares: auth'; then
  echo "    OK: TRAEFIK_MIDDLEWARE default present"
else
  fail "merged config missing traefik.http.routers.financegnome.middlewares: auth (TRAEFIK_MIDDLEWARE default)"
fi

echo "==> compose-config-check: TRAEFIK_HOST override"
override_config=$(TRAEFIK_HOST=staging.example.com compose -f docker-compose.yml \
  -f docker-compose.external.yml --profile external config)
if echo "$override_config" | grep -q 'staging.example.com'; then
  echo "    OK: TRAEFIK_HOST override renders"
else
  fail "TRAEFIK_HOST override not reflected in merged config"
fi

echo "==> compose-config-check: external overlay merged config (AC-2, T-0117)"
if ! echo "$config" | grep -q 'external: true'; then
  fail "traefik network must be external: true"
fi
for svc in flow-finance-ai grafana stats-forecast; do
  if ! echo "$config" | grep -A120 "^  ${svc}:" | grep -q 'traefik: null'; then
    fail "${svc} must join traefik network in external merge"
  fi
done
echo "    OK: flow-finance-ai, grafana, and stats-forecast on traefik network"

if echo "$config" | grep -A20 '^  flow-finance-ai:' | grep -qE '^    published: |^      published:'; then
  fail "flow-finance-ai must not publish host ports in external merge"
fi
if echo "$config" | grep -A20 '^  grafana:' | grep -qE '^    published: |^      published:'; then
  fail "grafana must not publish host ports in external merge (!reset)"
fi
echo "    OK: host ports cleared on flow-finance-ai and grafana"

if ! echo "$config" | grep -A120 '^  flow-finance-ai:' | grep -q 'DATABASE_HOST: postgres'; then
  fail "flow-finance-ai DATABASE_HOST must default to postgres in external merge"
fi
if ! echo "$config" | grep -A120 '^  flow-finance-ai:' | grep -q 'FIREFLY_BASE_URL: http://firefly:8080'; then
  fail "flow-finance-ai FIREFLY_BASE_URL must default to http://firefly:8080"
fi
if ! echo "$config" | grep -A120 '^  flow-finance-ai:' | grep -q 'FORECAST_ML_ENABLED: "false"'; then
  fail "flow-finance-ai FORECAST_ML_ENABLED must default to false in external merge"
fi
if ! echo "$config" | grep -A120 '^  flow-finance-ai:' | grep -q 'STATS_FORECAST_URL: http://stats-forecast:8090'; then
  fail "flow-finance-ai STATS_FORECAST_URL must default to http://stats-forecast:8090"
fi
echo "    OK: DNS override and ML env vars present"

if ! echo "$config" | grep -A40 '^  stats-forecast:' | grep -qE 'published: "8091"|published: 8091'; then
  fail "stats-forecast must publish host port 8091 in external merge"
fi
echo "    OK: stats-forecast host port remap 8091:8090"

for forbidden in postgres firefly firefly-iii; do
  if echo "$config" | grep -qE "^  ${forbidden}:"; then
    fail "forbidden service ${forbidden} must not appear in external merge"
  fi
done
echo "    OK: no postgres, firefly, or firefly-iii service definitions"

echo "==> compose-config-check: Grafana public router gated (AC-2)"
if echo "$config" | grep -A80 '^  grafana:' | grep -q 'traefik.enable: "true"'; then
  fail "grafana traefik.enable must not be true when GRAFANA_TRAEFIK_HOST unset"
fi
echo "    OK: Grafana Traefik disabled by default (empty GRAFANA_TRAEFIK_HOST)"

grafana_public=$(GRAFANA_TRAEFIK_HOST=grafana-financegnome.omniflow.cc \
  compose -f docker-compose.yml -f docker-compose.external.yml --profile external config)
if ! echo "$grafana_public" | grep -q 'grafana-financegnome.omniflow.cc'; then
  fail "GRAFANA_TRAEFIK_HOST override must render grafana router host"
fi
if ! echo "$grafana_public" | grep -A80 '^  grafana:' | grep -q 'traefik.enable: "true"'; then
  fail "grafana traefik.enable must be true when GRAFANA_TRAEFIK_HOST set"
fi
echo "    OK: optional Grafana public router wired when GRAFANA_TRAEFIK_HOST set"

echo "==> compose-config-check: all checks passed"
