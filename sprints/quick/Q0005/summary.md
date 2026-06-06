# Q0005 — Traefik auth loop + stale `index-CDBXTi4R.js`

## Symptoms

- Page loads but Traefik basic-auth dialog returns every ~5 seconds
- Console still shows `index-CDBXTi4R.js` (old bundle without BrowserRouter)
- `contentscript.js` warnings — browser extension, ignore

## Causes

1. **Auth loop:** Default React Query `refetchInterval: 5000` polled `/api/v1/*` every 5s. `fetch()` without `credentials: 'include'` did not reliably send cached Traefik basic-auth → **401 → browser re-prompts**.

2. **Stale JS:** Server serves `index-DsB7G24-.js` (fixed build) but browser cached old `index.html` / old chunk `index-CDBXTi4R.js`.

## Fixes

- `credentials: "include"` on all API `fetch` calls
- Removed global 5s refetch (pages that need polling set their own interval)
- `Cache-Control: no-cache` on `index.html`
- `AuthProvider` only when `VITE_OIDC_AUTHORITY` is set

## After deploy

1. Hard refresh or empty cache for `financegnome.omniflow.cc`
2. In DevTools → Network, confirm script is **`index-*.js` other than `CDBXTi4R`**
3. Traefik login should appear **once**, not every 5 seconds
