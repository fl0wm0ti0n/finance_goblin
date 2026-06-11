# Architecture archive pack (2026-06-11)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 12
- First archived heading: `# BUG-0020 — Subscriptions list quality (duplicates, uncategorized)`
- Last archived heading: `# BUG-0020 — Subscriptions list quality (duplicates, uncategorized)`
- Verification tuple (mandatory):
  - archived_body_lines=151
  - preamble_lines=10
  - retained_body_lines=2996

---

# BUG-0020 — Subscriptions list quality (duplicates, uncategorized)

**Status:** architecture complete (2026-06-11)  
**Discovery:** `discovery-20260610-bug0020` in `handoffs/po_to_tl.md`  
**Research:** [R-0090](research.md#r-0090--bug-0020-subscriptions-list-duplicates--uncategorized-display-category)  
**Decisions:** **DEC-0109** (reconcile + backfill + All-tab contract); extends **DEC-0085**, **DEC-0086**, **DEC-0100** (algorithms unchanged)  
**Acceptance:** `docs/product/acceptance.md` rows **BI** (DA), **BJ** (DB)

### Root cause (frozen, R-0090)

| Sub-defect | Symptom | Mechanism |
|------------|---------|-----------|
| **DA** | Strom 3×, YouTube 2× on **All** tab | **Strom:** 1 `rejected` + 2 `pending`, three `payee_key` variants (SEPA drift) — not confirmed dup. **YouTube:** 2 `confirmed` with divergent `payee_key` (pre-DEC-0084 debt). `list_patterns` has no dedup; All tab omits `status`. BUG-0015 prevents new pending for matching payee-interval but does not heal historical rows. |
| **DB** | All Category = Uncategorized | Migration 014 column-only; all confirms predated 014. `refresh_display_category_id` forward-only (DEC-0100 §3). 24/24 patterns have categorized linked txs; 0/24 `display_category_id` populated. |

`isolation_scope`: artifact + repo source reads; read-only DB/API probes per R-0090; no host `.env` / secrets read.

### Architecture contract (DEC-0109)

```text
BUG-0020
├── DA1 — one-time reconcile migration (P0)
│   └── 016_bug0020_subscription_list_quality.sql — YouTube confirmed merge; Strom pending collapse
├── DA2 — All-tab list contract (P0)
│   └── SubscriptionsPage.tsx — default exclude rejected + inactive from All visible set
├── DA3 — forward pending guard (P1)
│   └── detection.rs — skip pending when payee+interval matches confirmed/rejected (interval_matches)
├── DB1 — confirmed display_category backfill (P0)
│   └── same migration 016 — DEC-0100 RANK SQL batch UPDATE
└── T1 — integration tests + V1 operator smoke (P0)
    └── BI/BJ gates; discover/tags/detection/OIDC regression
```

### Files (frozen)

| Path | Role | Touch |
|------|------|-------|
| `backend/migrations/016_bug0020_subscription_list_quality.sql` | One-time reconcile + backfill | **DA1**, **DB1** |
| `backend/src/subscriptions/detection.rs` | Forward pending guard | **DA3** |
| `backend/src/subscriptions/repository.rs` | Optional `reconcile_cluster` helper for tests; reuse `refresh_display_category_id` in tests | **T1** |
| `frontend/src/pages/SubscriptionsPage.tsx` | All-tab visible filter | **DA2** |
| `backend/tests/bug0020_subscription_list_quality.rs` | BI/BJ integration tests | **T1** |

**No change:** `list_patterns` SQL shape (full-fidelity unfiltered API preserved); `compute_display_category_id` algorithm; discover/tags API; DEC-0100 refresh-on-confirm policy.

### Reconcile contract (DA1)

**YouTube cluster** (2 `confirmed`, same `display_name`, `interval_days` 30, `interval_matches`):

1. **Survivor** = row with earliest `confirmed_at` (preserves `subscription_alerts` history).
2. Relink `subscription_pattern_transactions` from loser → survivor (`ON CONFLICT DO NOTHING`, delete loser orphans).
3. Reassign `subscription_alerts.pattern_id` and `subscription_pattern_tags` to survivor.
4. Update survivor `payee_key`, `fingerprint`, `last_seen_at`, `current_amount` from fresher data.
5. Mark loser `status = 'inactive'`.

**Strom cluster** (2 `pending`, same `display_name`, `interval_days` 31):

1. **Survivor** = highest `last_seen_at`.
2. Relink txs; mark loser `status = 'rejected'`.
3. Existing `rejected` Strom row unchanged — hidden by DA2.

**Invariant:** Never merge rows where `interval_matches` is false (DEC-0086 multi-sub policy).

### Backfill contract (DB1)

Batch `UPDATE` all `status = 'confirmed'` patterns using DEC-0100 RANK SQL (identical to `repository.rs` `compute_display_category_id`). Expected post-backfill oracle (R-0090):

| `payee_key` (survivor) | `display_category_id` | Category name |
|------------------------|----------------------|---------------|
| netflix | 18 | Hobby & Freizeit - Streaming |
| kindle unltd | 18 | Hobby & Freizeit - Streaming |
| youtube (merged) | 66 | Reisen & Urlaub - Sonstiges |
| hgp unfall / hgp haushalt | 56 | Sonstiges - Versicherung Sonstiges |
| mitgliedsbeitrag - florian gabriel | 3 | Wohnen - Dienstleistungen |

Pending/rejected patterns remain NULL unless product extends scope later.

### All-tab contract (DA2)

| Tab | Visible statuses |
|-----|------------------|
| **All** | `pending`, `confirmed` |
| **Pending** | `pending` (unchanged — `?status=pending`) |
| **Standing** | `confirmed` + `kind=standing_order` (unchanged) |

Implementation: client-side filter on `listQuery` response when `tab === "all"`:

```typescript
patterns.filter((p) => p.status !== "rejected" && p.status !== "inactive")
```

API `GET /api/v1/subscriptions` without `status` remains unfiltered for operator/debug tooling.

### Forward guard (DA3)

Before `upsert_pending_pattern` in `run_candidates`, when no confirmed merge occurred:

- If `(group.payee_key, interval_days)` matches any **rejected** payee-interval → skip (existing).
- **Add:** if matches any **confirmed** payee-interval via `interval_matches` but merge returned `Ok(false)` (fingerprint conflict) → skip pending INSERT (log warn).

Prevents new Strom-style pending siblings after reconcile.

### Verification gates

| Gate | Proof |
|------|-------|
| **BI** | `GET /api/v1/subscriptions?status=confirmed` — ≤1 row per `payee_key`; All tab — no triplicate Strom / duplicate YouTube |
| **BJ** | netflix/kindle/youtube confirmed samples return non-null `display_category_id` matching R-0090 oracle |
| Discover/tags | `GET /api/v1/subscriptions/discover`, tag assign/filter unchanged |
| Detection | Post-fix sync — no new confirmed YouTube dup |
| OIDC | List endpoints smoke on omniflow profile |

**Pre-migration probe (operator/V1):**

```sql
SELECT id, status, payee_key, display_name, interval_days, confirmed_at
FROM subscription_patterns
WHERE display_name ILIKE '%strom%' OR display_name ILIKE '%youtube%'
ORDER BY display_name, status, confirmed_at;
```

**Post-migration probe:**

```sql
SELECT payee_key, COUNT(*) FROM subscription_patterns
WHERE status = 'confirmed' GROUP BY payee_key HAVING COUNT(*) > 1;
SELECT COUNT(*) FILTER (WHERE display_category_id IS NOT NULL), COUNT(*)
FROM subscription_patterns WHERE status = 'confirmed';
```

### Risks

| Risk | Mitigation |
|------|------------|
| Wrong YouTube survivor | Earliest `confirmed_at`; txn boundary |
| Multi-interval collapse | `interval_matches` gate only |
| Unexpected youtube→66 category | DEC-0100 contract; UAT documents |
| All-tab scope surprise | Release notes |
| Destructive migration | Backup before prod; dev rebuild acceptable |
| spt relink PK conflict | ON CONFLICT DO NOTHING + delete loser links |

### Rollback

`git revert` migration 016 + DA2/DA3 code. DB: restore backup or re-seed — reconcile marks `inactive`/`rejected` and relinks txs (not auto-reversible).

### Next phase

`/sprint-plan` — materialize `/quick` **Q0028** (recommended) from task table; then `/plan-verify` → `/execute`.

---

