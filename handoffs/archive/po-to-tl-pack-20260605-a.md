# PO to TL archive pack (2026-06-05)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 7
- First archived heading: `## intake-20260605-bug0008-0011 — Omniflow production regression batch 2 (4 operator issues)`
- Last archived heading: `## intake-20260605-bug0008-0011 — Omniflow production regression batch 2 (4 operator issues)`
- Verification tuple (mandatory):
  - archived_body_lines=52
  - retained_body_lines=485

---

## intake-20260605-bug0008-0011 — Omniflow production regression batch 2 (4 operator issues)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Bugs:** BUG-0008, BUG-0009, BUG-0010, BUG-0011 (+ US-0013, US-0014 epics)  
**Next phase:** `/discovery` on **BUG-0010** (recommended `/auto` P0 target)

### Summary

Operator report on `financegnome.omniflow.cc` (US-0010 external profile, **922+ transactions** synced, post-BUG-0004/0006 fixes) decomposed into **four bugs** and **two deferred US epics**. Overlap with **BUG-0004** (DONE — partial subscriptions/Grafana/forecast fixes) and **BUG-0007** (OPEN — AI chat enumeration) explicitly **related, not merged**.

| Bug | Priority | Sub-defects | Overlap decision |
|-----|----------|-------------|------------------|
| **BUG-0008** | P1 | W (33 alerts vs 11 list), X (under-detection) | Extends BUG-0004 J — new alert/list mismatch; coordinate BUG-0007 only |
| **BUG-0009** | P0 | Y (Grafana empty), Z (no account overview) | Post-BUG-0004 K/L regression/partial — separate Grafana surface |
| **BUG-0010** | P0 | AA (-25365.78 forecast), AB (wealth empty), AC (ML skipped) | Post-BUG-0004 L; epic → **US-0013** |
| **BUG-0011** | P1 | AD (empty plan no-op), AE (compare sums), AF (plan-vs-actual 404) | Supersedes BUG-0004 404 empty-state note; epic → **US-0014** |

### Intake evidence (US-0078)

| Bug | `intake_run_id` | Evidence bundle |
|-----|-----------------|-----------------|
| BUG-0008 | `intake-20260605-subscription-alerts-detection` | `handoffs/intake_evidence/intake-20260605-subscription-alerts-detection.json` |
| BUG-0009 | `intake-20260605-grafana-account-overview` | `handoffs/intake_evidence/intake-20260605-grafana-account-overview.json` |
| BUG-0010 | `intake-20260605-forecast-wealth-ml` | `handoffs/intake_evidence/intake-20260605-forecast-wealth-ml.json` |
| BUG-0011 | `intake-20260605-planning-mode-broken` | `handoffs/intake_evidence/intake-20260605-planning-mode-broken.json` |

All bundles: `small-intake-pack`, validation OK, `assumptions_confirmed: (none)`.

### Decomposition rationale

- **Split axis:** product surface (subscriptions alerts, Grafana embed, React forecast/wealth, planning UX)
- **Why not one BUG:** independent acceptance rows, different code paths, parallel `/auto` bug-queue candidates
- **Why US-0013/0014:** operator "implement fully" for ML forecast and intuitive planning exceeds quick defect scope

### Recommended `/auto` target

**BUG-0010** (P0 — wrong forecast **-25365.78** + empty wealth blocks core product value). Queue order suggestion: BUG-0010 → BUG-0009 → BUG-0008 → BUG-0011 → BUG-0007 (existing OPEN).

### Triad check (intake phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md` | BUG-0008–0011 + US-0013/0014 blocks | pass |
| `docs/product/acceptance.md` | W–AF sub-rows + US-0013/0014 criteria | pass |
| Intake evidence JSON | 4 bundles, distinct `topic_coverage` | pass |

`triad_hot_surface`: pending check after prepend

---

