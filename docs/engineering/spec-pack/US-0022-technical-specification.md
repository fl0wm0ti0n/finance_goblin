# Technical Specification — US-0022

## Overview

Full-stack deploy-observability: backend compile-time metadata endpoint, Docker build-arg propagation, Vite compile-time injection, subtle UI stamp, and on-mount stale detection. No new DEC; extends build pipeline and `AppLayout`.

## Components

| Layer | Change | Gate |
|-------|--------|------|
| `backend/src/meta/mod.rs` (new) | `BuildInfoResponse` struct + `build_info()` handler with `option_env!()` | B1 |
| `backend/src/api/mod.rs` | Register `mod meta;` + `.merge(meta::routes())` | B1 |
| `backend/Dockerfile` | Global `ARG BUILD_ID/RELEASE_TAG/BUILD_TIMESTAMP`; re-declare per stage; `ENV` in builder; `LABEL` in runtime | B2 |
| `frontend/vite.config.ts` | `define: { __BUILD_ID__: JSON.stringify(...), __RELEASE_TAG__: ... }` | F1 |
| `frontend/src/vite-env.d.ts` | `declare const __BUILD_ID__: string;` + `__RELEASE_TAG__` | F2 |
| `frontend/src/components/AppLayout.tsx` | Sidebar-footer stamp + tooltip | F3 |
| `frontend/src/hooks/useStaleDetection.ts` (new) | On-mount fetch + compare | F4 |
| `frontend/src/components/StaleBanner.tsx` (new) | Non-blocking banner + reload CTA | F5 |
| `backend/tests/meta_test.rs` (new or extend) | Integration: shape + content | T1 |

## Backend metadata endpoint (B1)

```rust
// backend/src/meta/mod.rs
use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
pub struct BuildInfoResponse {
    pub build_id: &'static str,
    pub release_tag: &'static str,
    pub build_timestamp: &'static str,
}

pub fn routes() -> Router {
    Router::new()
        .route("/api/v1/meta/build-info", get(build_info))
}

async fn build_info() -> Json<BuildInfoResponse> {
    Json(BuildInfoResponse {
        build_id: option_env!("BUILD_ID").unwrap_or("dev"),
        release_tag: option_env!("RELEASE_TAG").unwrap_or("dev"),
        build_timestamp: option_env!("BUILD_TIMESTAMP").unwrap_or("unknown"),
    })
}
```

**Registration in `api/mod.rs`:**

```rust
mod meta;
// ... in router chain:
.merge(meta::routes())
```

**Security:** Allowlist fields only. `option_env!()` never echoes `.env` or PAT. Public route (no auth) — metadata is non-sensitive.

## Dockerfile ARG chain (B2)

```dockerfile
# Global ARGs (before first FROM)
ARG BUILD_ID=unknown
ARG RELEASE_TAG=unknown
ARG BUILD_TIMESTAMP=unknown

FROM rust:1.88-bookworm AS builder
ARG BUILD_ID
ARG RELEASE_TAG
ARG BUILD_TIMESTAMP
# ... existing steps ...
ENV BUILD_ID=$BUILD_ID
ENV RELEASE_TAG=$RELEASE_TAG
ENV BUILD_TIMESTAMP=$BUILD_TIMESTAMP
RUN cargo build --release

FROM node:20-bookworm AS frontend
ARG BUILD_ID
ARG RELEASE_TAG
ARG BUILD_TIMESTAMP
# ... existing steps ...
RUN BUILD_ID=$BUILD_ID RELEASE_TAG=$RELEASE_TAG BUILD_TIMESTAMP=$BUILD_TIMESTAMP npm run build

FROM debian:bookworm-slim
# ... existing runtime setup ...
LABEL org.opencontainers.image.revision="${BUILD_ID}"
LABEL org.opencontainers.image.version="${RELEASE_TAG}"
LABEL org.opencontainers.image.created="${BUILD_TIMESTAMP}"
```

**CI invocation:**

```bash
docker build \
  --build-arg BUILD_ID=$(git rev-parse --short HEAD) \
  --build-arg RELEASE_TAG=$(cat handoffs/releases/latest-tag 2>/dev/null || echo "dev") \
  --build-arg BUILD_TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ) \
  -t flow-finance-ai:latest \
  -f backend/Dockerfile .
```

## Vite define injection (F1)

```typescript
// frontend/vite.config.ts
export default defineConfig({
  plugins: [react()],
  define: {
    __BUILD_ID__: JSON.stringify(process.env.BUILD_ID || 'dev'),
    __RELEASE_TAG__: JSON.stringify(process.env.RELEASE_TAG || 'dev'),
  },
  // ... existing config
});
```

## TypeScript declarations (F2)

```typescript
// frontend/src/vite-env.d.ts
declare const __BUILD_ID__: string;
declare const __RELEASE_TAG__: string;
```

## Stale detection hook (F4)

```typescript
// frontend/src/hooks/useStaleDetection.ts
import { useEffect, useState } from 'react';

interface BuildInfo {
  build_id: string;
  release_tag: string;
  build_timestamp: string;
}

export function useStaleDetection() {
  const [stale, setStale] = useState(false);
  const [serverInfo, setServerInfo] = useState<BuildInfo | null>(null);

  useEffect(() => {
    if (__BUILD_ID__ === 'dev') return;
    fetch('/api/v1/meta/build-info', { cache: 'no-store' })
      .then(r => r.json())
      .then((info: BuildInfo) => {
        setServerInfo(info);
        if (info.build_id !== __BUILD_ID__) setStale(true);
      })
      .catch(() => {});
  }, []);

  return { stale, serverInfo };
}
```

## UI integration (F3, F5)

| Element | Placement | Behavior |
|---------|-----------|----------|
| Version stamp | `AppLayout` sidebar-footer | Short build id or release tag fragment; always visible |
| Tooltip | On hover/focus of stamp | Release tag + build id + build timestamp (UTC) |
| Stale banner | Top of app (above content) | Non-blocking; "New version available — reload"; dismissible; hidden when `!stale` |

## Non-functional

- **Compatibility:** localhost `:18080`, omniflow; backend + frontend co-deploy
- **Testing:** Rust integration test for meta endpoint shape; `cargo test`; `npm test`; frontend build
- **Deploy:** Backend + frontend rebuild; no migration; Docker `--build-arg` chain
- **Security:** Allowlist fields only; no auth required; no secrets exposure

## Traceability

- [R-0095](docs/engineering/research.md#r-0095--us-0022-deploy-version-stamp--stale-frontend-detection) §6–§12
- `docs/engineering/architecture.md` § US-0022
- No new DEC (GATE-DEC-1 closed)
