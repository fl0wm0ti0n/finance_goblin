#!/usr/bin/env bash
# Wrapper: run omniflow deploy from canonical host path /workdir/financegoblin
set -euo pipefail
exec /workdir/financegoblin/deploy.sh "$@"
