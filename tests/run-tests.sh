#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> Backend unit tests"
cd backend
cargo test --test firefly_readonly
cargo test --lib

if [[ -n "${DATABASE_URL:-}" ]]; then
  echo "==> Firefly read-only integration test"
  cargo test --test firefly_integration
  echo "==> Forecast integration test"
  cargo test --test forecast_integration
  echo "==> Subscription integration test"
  cargo test --test subscriptions_integration
  echo "==> Plan integration test"
  cargo test --test plans_integration
  echo "==> Wealth/alerts integration test"
  cargo test --test wealth_alerts_integration
  echo "==> AI assistant integration test"
  cargo test --test ai_assistant_integration
  echo "==> Exchange portfolio integration test"
  cargo test --test exchanges_portfolio_integration
  cargo test --test exchange_signing
else
  echo "==> SKIP firefly_integration (set DATABASE_URL for full integration test)"
  echo "==> SKIP forecast_integration (set DATABASE_URL for full integration test)"
  echo "==> SKIP subscriptions_integration (set DATABASE_URL for full integration test)"
  echo "==> SKIP plans_integration (set DATABASE_URL for full integration test)"
  echo "==> SKIP wealth_alerts_integration (set DATABASE_URL for full integration test)"
  echo "==> SKIP ai_assistant_integration (set DATABASE_URL for full integration test)"
  echo "==> SKIP exchanges_portfolio_integration (set DATABASE_URL for full integration test)"
fi

if [[ -n "${DATABASE_BOOTSTRAP_TEST_URL:-}" ]]; then
  echo "==> Database bootstrap integration test (US-0012)"
  cargo test --test database_bootstrap_integration
else
  echo "==> SKIP database_bootstrap_integration (set DATABASE_BOOTSTRAP_TEST_URL for AC-6 fixture)"
fi

cargo test --test exchange_signing
echo "==> AI local provider isolation (wiremock)"
cargo test --test ai_local_provider_isolation
cargo test --test ai_frozen_modules
cargo test --test forecast_ml_integration
echo "==> Grafana provisioning BUG-0009 contract test"
cargo test --test grafana_provisioning_bug0009
echo "==> Analytics proxy integration test"
cargo test --test analytics_proxy_integration
echo "==> Product routes regression test"
cargo test --test product_routes_regression

cd "$ROOT"

echo "==> Frontend build"
cd "$ROOT/frontend"
if [[ "${AUTO_INSTALL_DEPS:-0}" == "1" ]] || [[ ! -d node_modules/@tanstack ]]; then
  npm install
fi
if [[ "${AUTO_INSTALL_DEPS:-0}" == "1" ]] || [[ ! -d node_modules/echarts ]]; then
  npm install
fi
npm run build

echo "==> Compose config regression (US-0010)"
bash "$ROOT/scripts/compose-config-check.sh"

echo "==> All tests passed"
