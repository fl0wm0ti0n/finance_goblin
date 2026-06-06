# Q0004 — React crash after Traefik login

## Symptoms

After Traefik basic-auth, browser console:

```text
Uncaught Error at de (index-*.js:50:361)
```

Stack includes React Router internals (`si()||de(!1)`).

## Cause

`frontend/src/main.tsx` mounted `<App />` with `Routes` / `useLocation` / `useNavigate` but **no `BrowserRouter`**. React Router throws when router hooks run outside a router context.

OIDC and API were unrelated; Traefik auth only exposed the broken SPA.

## Fix

```tsx
import { BrowserRouter } from "react-router-dom";
// ...
<BrowserRouter>
  <App />
</BrowserRouter>
```

Rebuild and redeploy `flow-finance-ai`.

## Operator

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Hard refresh `https://financegnome.omniflow.cc/`.
