# Sprint S0021 — Tasks

## Task list

| ID | Title | Slice | Est. | Acceptance | Status |
|----|-------|-------|------|------------|--------|
| B1 | Backend `meta` module + route registration | S1 | 3h | AC-3 | TODO |
| B2 | Dockerfile `ARG`/`ENV`/`LABEL` chain | S2 | 2h | AC-4 | TODO |
| F1 | Vite `define` block injection | S3 | 1h | AC-4 | TODO |
| F2 | TypeScript declarations (`vite-env.d.ts`) | S3 | 0.5h | AC-4 | TODO |
| F3 | `AppLayout` sidebar-footer stamp + tooltip | S4 | 3h | AC-1, AC-2 | TODO |
| F4 | `useStaleDetection` hook | S4 | 2h | AC-5 | TODO |
| F5 | `StaleBanner` component | S4 | 2h | AC-5 | TODO |
| T1 | Integration test — meta endpoint shape | S1 | 1.5h | AC-3 | TODO |
| G1 | Automated gate — cargo test + npm test + build | S5 | 1h | all | TODO |
| R1 | User guide US-0022 | S5 | 2h | — | TODO |
| V1 | Verify-work AC-1..AC-6 + OIDC smoke | S5 | 2h | AC-6 | TODO |

## Task details

### B1 — Backend `meta` module + route registration

**Files:**
- `backend/src/meta/mod.rs` (new)
- `backend/src/api/mod.rs` (edit)

**Acceptance:** AC-3

**Implementation:**
- Create `BuildInfoResponse` struct with `build_id`, `release_tag`, `build_timestamp` fields (all `&'static str`)
- Implement `build_info()` handler using `option_env!("BUILD_ID").unwrap_or("dev")`, `option_env!("RELEASE_TAG").unwrap_or("dev")`, `option_env!("BUILD_TIMESTAMP").unwrap_or("unknown")`
- Register route: `pub fn routes() -> Router { Router::new().route("/api/v1/meta/build-info", get(build_info)) }`
- In `api/mod.rs`: add `mod meta;` and `.merge(meta::routes())` to router chain
- Public route (no auth) — metadata is non-sensitive
- Allowlist fields only — never echo `.env` or PAT

**Gate:** GATE-META-1 (dedicated `/api/v1/meta/build-info`, not extend `/health`)

---

### B2 — Dockerfile `ARG`/`ENV`/`LABEL` chain

**Files:**
- `backend/Dockerfile` (edit)

**Acceptance:** AC-4

**Implementation:**
- Add global `ARG` declarations before first `FROM`:
  ```dockerfile
  ARG BUILD_ID=unknown
  ARG RELEASE_TAG=unknown
  ARG BUILD_TIMESTAMP=unknown
  ```
- In `builder` stage (Rust):
  - Re-declare `ARG BUILD_ID`, `ARG RELEASE_TAG`, `ARG BUILD_TIMESTAMP`
  - Convert to `ENV`: `ENV BUILD_ID=$BUILD_ID`, `ENV RELEASE_TAG=$RELEASE_TAG`, `ENV BUILD_TIMESTAMP=$BUILD_TIMESTAMP`
  - Place after dependency `COPY` to minimize cache invalidation
- In `frontend` stage (Node):
  - Re-declare `ARG BUILD_ID`, `ARG RELEASE_TAG`, `ARG BUILD_TIMESTAMP`
  - Pass to build: `RUN BUILD_ID=$BUILD_ID RELEASE_TAG=$RELEASE_TAG BUILD_TIMESTAMP=$BUILD_TIMESTAMP npm run build`
- In runtime stage (Debian):
  - Re-declare `ARG BUILD_ID`, `ARG RELEASE_TAG`, `ARG BUILD_TIMESTAMP`
  - Add OCI labels:
    ```dockerfile
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

**Gate:** GATE-BUILD-1 (git short sha + release tag + UTC timestamp)

---

### F1 — Vite `define` block injection

**Files:**
- `frontend/vite.config.ts` (edit)

**Acceptance:** AC-4

**Implementation:**
- Add `define` block to `defineConfig`:
  ```typescript
  define: {
    __BUILD_ID__: JSON.stringify(process.env.BUILD_ID || 'dev'),
    __RELEASE_TAG__: JSON.stringify(process.env.RELEASE_TAG || 'dev'),
  }
  ```
- Values are static text replacements — must use `JSON.stringify()` for string literals
- Works in both dev and production; `.env` vars only work when `.env` file present

**Gate:** architecture (Vite `define` pattern, not `import.meta.env.VITE_*`)

---

### F2 — TypeScript declarations (`vite-env.d.ts`)

**Files:**
- `frontend/src/vite-env.d.ts` (edit)

**Acceptance:** AC-4

**Implementation:**
- Add declarations:
  ```typescript
  declare const __BUILD_ID__: string;
  declare const __RELEASE_TAG__: string;
  ```

**Gate:** architecture (TypeScript type safety for build-time constants)

---

### F3 — `AppLayout` sidebar-footer stamp + tooltip

**Files:**
- `frontend/src/components/AppLayout.tsx` (edit)

**Acceptance:** AC-1, AC-2

**Implementation:**
- In `sidebar-footer` (lines 78-91), add subtle version stamp:
  - Default: short build id fragment (e.g. `__BUILD_ID__.slice(0, 7)`) or release tag
  - Always visible; low visual noise
- On hover/focus, show tooltip with:
  - Release tag (`__RELEASE_TAG__`)
  - Build id (`__BUILD_ID__`)
  - Build timestamp (UTC) — fetch from `/api/v1/meta/build-info` or embed at compile time
- Use existing tooltip component or `title` attribute
- Placement: below OIDC user name + logout (natural location for operator-only stamp)

**Gate:** GATE-UI-1 (sidebar footer, subtle by default)

---

### F4 — `useStaleDetection` hook

**Files:**
- `frontend/src/hooks/useStaleDetection.ts` (new)

**Acceptance:** AC-5

**Implementation:**
```typescript
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
    const clientBuildId = __BUILD_ID__;
    if (clientBuildId === 'dev') return; // skip in dev mode

    fetch('/api/v1/meta/build-info', { cache: 'no-store' })
      .then(r => r.json())
      .then((info: BuildInfo) => {
        setServerInfo(info);
        if (info.build_id !== clientBuildId) {
          setStale(true);
        }
      })
      .catch(() => {/* silent fail — non-blocking */});
  }, []);

  return { stale, serverInfo };
}
```

**Behavior:**
- On mount: fetch `/api/v1/meta/build-info` with `cache: 'no-store'`
- Compare `__BUILD_ID__` to server `build_id`
- Mismatch → `stale=true`
- Skip when `__BUILD_ID__ === 'dev'` (local dev)
- Silent fail on network error (`.catch(() => {})`) — non-blocking

**Gate:** GATE-STALE-1 (on-mount fetch only, no periodic poll)

---

### F5 — `StaleBanner` component

**Files:**
- `frontend/src/components/StaleBanner.tsx` (new)

**Acceptance:** AC-5

**Implementation:**
- Non-blocking banner at top of app (above content)
- Message: "New version available — reload"
- Reload button/CTA
- Dismissible (optional)
- Hidden when `!stale`
- Use `useStaleDetection()` hook

**Integration:**
- In `AppLayout.tsx`, render `<StaleBanner />` above main content
- Pass `stale` state from `useStaleDetection()`

**Gate:** architecture (non-blocking, dismissible)

---

### T1 — Integration test — meta endpoint shape

**Files:**
- `backend/tests/meta_test.rs` (new or extend)

**Acceptance:** AC-3

**Implementation:**
- Test `GET /api/v1/meta/build-info` returns 200
- Test response shape: `{build_id, release_tag, build_timestamp}`
- Test no secrets in response (no `.env` values, no PAT)
- Test `option_env!()` fallback: when `BUILD_ID` not set, returns `"dev"`

**Gate:** GATE-TEST-1 (integration test for meta endpoint)

---

### G1 — Automated gate — cargo test + npm test + build

**Files:**
- (no files — automated gate)

**Acceptance:** all

**Implementation:**
- Run `cargo test --lib` — all tests pass
- Run `cargo test --test meta_test` — meta endpoint tests pass
- Run `npm test` — all frontend tests pass
- Run `npm run build` — frontend builds successfully with `__BUILD_ID__` and `__RELEASE_TAG__` injected

**Gate:** architecture (automated quality gate)

---

### R1 — User guide US-0022

**Files:**
- `docs/user-guides/US-0022.md` (new)

**Acceptance:** —

**Implementation:**
- Document version stamp location (sidebar footer)
- Document hover tooltip (release tag + build id + build timestamp)
- Document stale banner behavior (mismatch → reload CTA)
- Document expected behavior after backend-only deploy (banner appears until frontend rebuild)
- Document hard refresh hint in tooltip (Traefik/browser cache)

**Gate:** `USER_GUIDE_MODE=1` (user guide required)

---

### V1 — Verify-work AC-1..AC-6 + OIDC smoke

**Files:**
- `sprints/S0021/uat.json` (edit)
- `sprints/S0021/uat.md` (edit)

**Acceptance:** AC-6

**Implementation:**
- AC-1: Subtle stamp visible in sidebar footer; does not dominate primary UX
- AC-2: Hover tooltip reveals release tag + build id + build timestamp (UTC)
- AC-3: `GET /api/v1/meta/build-info` returns `{build_id, release_tag, build_timestamp}`; no secrets
- AC-4: Frontend bundle embeds build id at compile time (verify via `npm run build` + inspect bundle or `__BUILD_ID__` in DOM)
- AC-5: On-mount stale detection: mismatch → banner + reload CTA; match → no banner
- AC-6: `/health` liveness unchanged (`{status: ok}`); OIDC external profile smoke pass; no env secrets in metadata

**Operator gate:** **BACKEND_FRONTEND_DEPLOY** → verify-work omniflow smoke (UAT)

**Gate:** architecture (verify-work + OIDC smoke)
