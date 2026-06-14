# PO to TL archive pack (2026-06-13)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 6
- Retained units in hot file: 40
- First archived heading: `## Operator intent`
- Last archived heading: `## Risks`
- Verification tuple (mandatory):
  - archived_body_lines=39
  - retained_body_lines=357

---

## Operator intent

After BUG-0023 deploy confusion (code released `bug0023-q0030` but live container still pre-2026-06-09), operator needs an **in-app oracle** for deployed version — not `docker inspect`.

> „die genaue version, build etc soll auf der webpage irgendwo versteckt mit hover für mehr infos stehen … eventuell brauchen wir auch einen algo der prüft ob die im browser geladen version noch alt ist, eventuell via backend abgleich?“

## Scope summary

| In | Out |
|----|-----|
| Subtle footer/sidebar stamp + hover tooltip (release tag, build id, UTC time) | Full release-management UI |
| Backend authoritative build metadata (`/health` extend or `/api/v1/meta/build-info`) | Semver auto-bump |
| Vite `VITE_BUILD_ID` / release tag at Docker build | Secrets in metadata |
| Non-blocking stale SPA banner when bundle id ≠ backend | |

## Acceptance (AC-1..AC-6)

See `docs/product/acceptance.md` § US-0022.

## Research

[R-0095](docs/engineering/research.md#r-0095--us-0022-deploy-version-stamp--stale-frontend-detection) — patterns for build stamps, `/health` extension vs meta endpoint, Vite define injection, stale-bundle detection.

## Architecture questions (carry to `/discovery`)

1. **GATE-META-1:** Extend `/health` vs new `/api/v1/meta/build-info`?
2. **GATE-BUILD-1:** Single `BUILD_ID` (git sha) vs composite (release tag + image digest fragment)?
3. **GATE-STALE-1:** Check on mount only vs poll every N minutes?
4. **GATE-UI-1:** Footer text vs sidebar dot — placement in `AppLayout.tsx`?

## Risks

- Traefik/browser cache serves old `index.html` after deploy — stale banner is mitigation; tooltip should mention hard refresh
- Backend-only deploy without frontend rebuild → id mismatch until both rebuilt (expected; banner explains)

**Recommended next phase:** `/discovery` (US-0022)

---

