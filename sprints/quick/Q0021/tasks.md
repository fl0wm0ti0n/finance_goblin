# Tasks — Q0021 (US-0017)

**Story:** US-0017  
**Task count:** 7 (E1–E6 + UG1; < `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260609-q0021-us0017`

## Architecture → sprint mapping

| Architecture ID | Task | Notes |
|-----------------|------|-------|
| **E1** | Task **E1** | `### Omniflow smoke (external profile)` under `## Examples`; R-0078 §2 |
| **E2** | Task **E2** | `### Troubleshooting` under `## Limitations`; R-0078 §3 / Q0020 uat |
| **E3** | Task **E3** | Verify-only Product status; append only if new segment closures |
| **E4** | Task **E4** | `docs/developer/README.md` per-segment wording (R-0078 §5) |
| **E5** | Task **E5** | Runbook § README maintenance + release-segment definition |
| **E6** | Task **E6** | `validate_doc_profile --no-template-parity` fail-closed gate |
| — | Task **UG1** | `USER_GUIDE_MODE=1` — `docs/user-guides/US-0017.md` |

## Execute order

```text
E1 ∥ E2 ∥ E4 ∥ E5
  → E3
  → UG1
  → E6
```

**Parallelism:** E1, E2, E4, E5 touch disjoint files and may proceed in parallel; E6 blocked on all prior tasks.

## Acceptance traceability

| AC | Tasks | Verify |
|----|-------|--------|
| **AC-1** | E1, UG1 | Omniflow external-profile smoke H3 + operator guide distill |
| **AC-2** | E2, UG1 | Troubleshooting H3 + ML-off vs data-missing distinction |
| **AC-3** | E3 | Product status verify — US-0015 + post-US-0016 closures |
| **AC-4** | E4, E5 | Developer README + runbook per-segment maintenance hooks |
| **AC-5** | E6 | Validator exit 0; split layout preserved |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| E1 | Omniflow smoke H3 under Examples | 1.5h | open | **AC-1** | P0 |
| E2 | Troubleshooting H3 under Limitations | 1.5h | open | **AC-2** | P0 |
| E3 | Product status verify-only | 0.5h | open | **AC-3** | P0 |
| E4 | Developer README per-segment wording | 0.5h | open | **AC-4** | P0 |
| E5 | Runbook § README maintenance delta | 0.5h | open | **AC-4** | P0 |
| UG1 | Publish `docs/user-guides/US-0017.md` | 0.5h | open | **AC-1**, **AC-2** | P0 |
| E6 | `validate_doc_profile --no-template-parity` gate | 0.5h | open | **AC-5** | P0 |

---

## E1 — Omniflow smoke H3 under Examples

**Status:** open  
**Depends on:** —  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` US-0017 **AC-1**

### Description

Add `### Omniflow smoke (external profile)` under `## Examples` in root `README.md` per **DEC-0070** US-0017 extension and R-0078 §2:

- Host `https://financegnome.omniflow.cc` (note `TRAEFIK_HOST` override)
- Traefik edge auth: placeholder `-u '<basic-auth-user>:<pass>'` — **never** commit credentials
- API auth one-liner: OIDC session or `AUTH_DEV_BYPASS=true`; matrix in runbook
- Copy-paste block: health, sync status/entities, `POST /api/v1/sync/trigger` full mode, `GET /api/v1/forecast/meta`, `GET /api/v1/wealth` crypto probe, Grafana embed health
- Six `/analytics/{slug}` smoke focuses (extend existing table or `OMNI` prefix note)
- Operator gates one-liner: **BACKEND_FRONTEND_DEPLOY** → **GRAFANA_PROVISIONING_RELOAD** → **FULL_FIREFLY_SYNC** + recompute; link runbook §23

**Files:** `README.md`

### Done when

- [ ] H3 present under `## Examples` with external-profile curls (not localhost-only)
- [ ] Sync trigger, forecast recompute pointer, analytics routes, exchange sanity documented
- [ ] Traefik placeholder auth pattern; no secret literals
- [ ] Gate sequence one-liner with runbook deep link

---

## E2 — Troubleshooting H3 under Limitations

**Status:** open  
**Depends on:** —  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` US-0017 **AC-2**

### Description

Add `### Troubleshooting` under `## Limitations` (not a new root H2) per R-0078 §3:

- Lead: operator gate sequence (same three gates as E1)
- Body: 6-row symptom table from Q0020 / BUG-0013 discovery (flat analytics, MTD artifact, crypto €0, forecast 0 €, ML unavailable banner, Grafana Failed to fetch)
- Distinction: empty Grafana SQL panels after gates = data/deploy defect; **ML unavailable** banner = expected degraded mode (US-0013 / DEC-0049)
- Deep link: runbook §23 for row-level detail

**Files:** `README.md`

### Done when

- [ ] H3 under `## Limitations` with gate sequence + symptom table
- [ ] ML-unavailable vs empty-panel distinction explicit
- [ ] `BACKEND_FRONTEND_DEPLOY` cadence documented
- [ ] Sync+recompute prerequisite before attributing analytics failures to code
- [ ] No new root `## Troubleshooting` H2

---

## E3 — Product status verify-only

**Status:** open  
**Depends on:** E1, E2  
**Estimate:** 0.5h  
**Acceptance hook:** `docs/product/acceptance.md` US-0017 **AC-3**

### Description

Verify `### Product status` under `## Purpose` already lists **US-0015**, **BUG-0013**, and **US-0013–0016** (post-Q0020 refresh-context). Append bullets **only** if the release segment closes additional US/BUG ids before story close.

**Files:** `README.md` (verify; minimal edit if needed)

### Done when

- [ ] Product status includes US-0015 and post-US-0016 closures
- [ ] No duplicate or stale bullets introduced
- [ ] Document verify outcome in execute summary if no edits required

---

## E4 — Developer README per-segment wording

**Status:** open  
**Depends on:** —  
**Estimate:** 0.5h  
**Acceptance hook:** `docs/product/acceptance.md` US-0017 **AC-4**

### Description

Tighten `docs/developer/README.md` Quality gates / Workflow sections: require Product status README update for **each** closed US/BUG in the **current release segment** (sprint id `Sxxxx`, quick id `Qxxxx`, or paired intake batch). Pointer to runbook § README maintenance.

**Files:** `docs/developer/README.md`

### Done when

- [ ] Per-segment wording replaces vague "closed items" language
- [ ] Release + refresh-context checklists reference each closed US/BUG in segment
- [ ] Runbook § README maintenance cross-linked

---

## E5 — Runbook § README maintenance delta

**Status:** open  
**Depends on:** —  
**Estimate:** 0.5h  
**Acceptance hook:** `docs/product/acceptance.md` US-0017 **AC-4**

### Description

Update `docs/engineering/runbook.md` § README maintenance (US-0016):

- Define **release segment** = target sprint id, quick task id, or paired intake batch for current `/release` or `/refresh-context`
- Release hook: for **each** US/BUG → DONE/CLOSED in segment, append one Product status bullet
- Refresh hook: when segment closed one or more US/BUG since prior refresh, verify **each** closed id appears in Product status

**Files:** `docs/engineering/runbook.md`

### Done when

- [ ] Release-segment definition documented
- [ ] Per-segment release and refresh hooks explicit
- [ ] Aligns with DEC-0070 phase-boundary cadence

---

## UG1 — Publish operator user guide

**Status:** open  
**Depends on:** E1, E2  
**Estimate:** 0.5h  
**Acceptance hook:** US-0017 **AC-1**, **AC-2** (operator distill; `USER_GUIDE_MODE=1`)

### Description

Create `docs/user-guides/US-0017.md` — operator-facing quick reference for omniflow smoke and troubleshooting distilled from E1/E2 README H3 sections. Cross-link root README, runbook §23, and related user guides. No credential literals.

**Files:** `docs/user-guides/US-0017.md`

### Done when

- [ ] User guide published with omniflow smoke + troubleshooting sections
- [ ] Links to README H3 anchors and runbook §23
- [ ] Gate sequence and symptom table summarized for operators

---

## E6 — Validator gate

**Status:** open  
**Depends on:** E1, E2, E3, E4, E5, UG1  
**Estimate:** 0.5h  
**Acceptance hook:** `docs/product/acceptance.md` US-0017 **AC-5**

### Description

Run `python scripts/validate_doc_profile.py --repo . --no-template-parity` after all doc edits. Non-zero exit → fail closed; remediate per runbook § README maintenance. Confirm: no `DEV_*` H2 in root; H2 budget ≤ 8; split layout preserved.

**Files:** (validator run only — no CI structural changes)

### Done when

- [ ] `validate_doc_profile --no-template-parity` exits **0**
- [ ] No new forbidden root H2
- [ ] H3 additions only under existing H2s
