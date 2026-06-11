# Architecture archive pack (2026-06-09)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 10
- First archived heading: `# BUG-0017 — Post-sync forecast recompute cluster`
- Last archived heading: `# BUG-0017 — Post-sync forecast recompute cluster`
- Verification tuple (mandatory):
  - archived_body_lines=165
  - preamble_lines=10
  - retained_body_lines=2930

---

# BUG-0017 — Post-sync forecast recompute cluster

**Status:** architecture complete (2026-06-09)  
**Discovery:** `discovery-20260609-bug0017` in `handoffs/archive/po-to-tl-pack-20260609-h.md`  
**Research:** [R-0087](research.md#r-0087--bug-0017-post-sync-forecast-recompute-cluster-audit-check-fk-retention-ml-gate-forecastpage-loading)  
**Decisions:** **DEC-0105** (audit CHECK extension); **DEC-0106** (`paired_baseline_id` CASCADE + retention order); extends **DEC-0034**, **DEC-0078**, **DEC-0011**, **DEC-0050**, **DEC-0052**  
**Sprint:** `/quick` (PLANNED — ≤6 tasks)  
**Acceptance:** `docs/product/acceptance.md` rows **AY–BD**  
**Spec-pack:** `docs/engineering/spec-pack/BUG-0017-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**Related:** UI-002, UI-006, UI-009, UI-010; US-0015 bucket audit; BUG-0010/BUG-0014 prior forecast posture

### Root-cause chain (frozen)

Post-sync pipeline per [R-0050](research.md#r-0050--sync-mutex-ml-phase-integration-and-history-gates):

```text
sync → subscriptions → forecast (baseline) → plan hook → forecast_ml → exchanges/alerts
```

| AC | Defect | Mechanism | Symptom |
|----|--------|-----------|---------|
| **AY** | `ai_tool_audit_tool_name_check` | `006_ai_audit.sql` allows six chat tools only; US-0015 inserts `forecast_bucket_assignment` | WARN audit insert; row dropped |
| **AZ** | `ai_tool_audit_result_status_check` | CHECK allows `ok`/`error` only; bucket inference emits extended statuses | WARN audit insert; row dropped |
| **BA** | `forecast_computations_paired_baseline_id_fkey` | `enforce_retention()` deletes baseline while `ml_enhanced` rows reference `paired_baseline_id` (`NO ACTION`) | Recompute `Err`; sync serves stale baseline |
| **BB** | ML `insufficient_history` | Gate: max monthly net-cashflow points per account &lt; `min_monthly_points` (default 12) | ML buttons disabled; verify after **BA** |
| **BC** | Plan stale (downstream) | Plan hook runs only on successful baseline recompute | **Plan stale** badge persists |
| **BD** | Forecast empty flash (UX) | `emptyState = !metaQuery.data?.computation_id` while query pending | Brief **No forecast data yet** |

Sync remains **success** on recompute `Err` — by design per R-0050 (gate 7 **deferred**).

`isolation_scope`: artifact + repo source reads; no host `.env` / `.env_prod` secrets read.

### Architecture contract

```text
BUG-0017
├── AY1 — ai_tool_audit CHECK migration (P0)
│   └── DEC-0105: extend tool_name + result_status CHECK (NOT VALID + VALIDATE)
├── BA1 — FK CASCADE migration (P0)
│   └── DEC-0106: paired_baseline_id ON DELETE CASCADE
├── BA2 — Retention loop order (P0)
│   └── enforce_retention: ml_enhanced before baseline (defense in depth)
├── BD1 — ForecastPage loading guard (P0)
│   └── empty only when isFetched && !isError && !computation_id
├── T1 — Retention integration test (P0)
│   └── Paired baseline+ML rows prune without FK error
└── V1 — verify-work operator smoke (P0)
    └── Sync logs clean; meta fresh; BB month-bucket SQL; BC planning; BD nav
```

**Deploy:** Backend migration(s) + `repository.rs` + `ForecastPage.tsx`; rebuild `flow-finance-ai` image post-Q0024.

### DEC-0105 — Audit CHECK extension

New migration `0XX_bug0017_ai_audit_forecast.sql`:

| Column | Additions |
|--------|-----------|
| `tool_name` | `forecast_bucket_assignment` |
| `result_status` | `low_confidence`, `provider_unavailable`, `parse_error` |

Pattern: DROP CONSTRAINT → ADD with `NOT VALID` → `VALIDATE CONSTRAINT` ([R-0087 §2](research.md#r-0087--bug-0017-post-sync-forecast-recompute-cluster-audit-check-fk-retention-ml-gate-forecastpage-loading)).

**Rejected:** status mapping in Rust (loses fidelity); PostgreSQL ENUM; separate audit table.

### DEC-0106 — FK retention policy

```sql
ALTER TABLE forecast_computations
  DROP CONSTRAINT forecast_computations_paired_baseline_id_fkey,
  ADD CONSTRAINT forecast_computations_paired_baseline_id_fkey
    FOREIGN KEY (paired_baseline_id) REFERENCES forecast_computations(id)
    ON DELETE CASCADE NOT VALID;
ALTER TABLE forecast_computations VALIDATE CONSTRAINT forecast_computations_paired_baseline_id_fkey;
```

Retention loop order frozen:

```rust
for kind in ["ml_enhanced", "baseline"] { /* enforce_retention */ }
```

**Rejected alone:** ml-first reorder without CASCADE; `ON DELETE SET NULL`; application-only child DELETE as sole fix.

**BC:** Downstream of **BA** — verify-work Planning Compare re-smoke only; no plan-engine change.

### BD — ForecastPage loading contract

```tsx
const showLoading = metaQuery.isPending;
const showEmpty =
  metaQuery.isFetched && !metaQuery.isError && !metaQuery.data?.computation_id;
```

Show skeleton when `showLoading`; empty card when `showEmpty`; preserve content when `computation_id` present.

**Rejected:** `placeholderData` (defer); SSR `initialData` (no SSR).

### BB — ML gate verification (operator)

Post-deploy month-bucket probe — do **not** lower `min_monthly_points`:

```sql
SELECT account_id,
       COUNT(DISTINCT date_trunc('month', date)) AS month_buckets
FROM transactions WHERE date IS NOT NULL
GROUP BY account_id ORDER BY month_buckets;
```

| Outcome | Action |
|---------|--------|
| All asset accounts ≥ 12 months | Gate should pass after **BA**; else investigate sidecar path |
| Any account &lt; 12 months | Honest `ml_skipped_reason` — BB satisfied |

### Deferred gates

| Gate | Resolution |
|------|------------|
| Sync fail-on-recompute | **Defer** — preserve R-0050 warn-and-serve; product decision gate |
| BC separate fix | **Reject** — downstream of **BA** |

### Operator smoke matrix (post-fix)

1. `POST /api/v1/sync/trigger` — logs free of audit CHECK WARN and FK WARN
2. `GET /api/v1/forecast/meta` — fresh `computation_id`, `stale=false`
3. `SELECT … FROM ai_tool_audit WHERE tool_name = 'forecast_bucket_assignment'` — rows present
4. ML: `ml_computation_id` set OR honest `ml_skipped_reason` after month-bucket probe
5. Planning Compare — no **Plan stale** after successful recompute
6. Forecast nav from Home — no false empty when meta has `computation_id`

### Risks

| Risk | Mitigation |
|------|------------|
| Constraint name drift on operator DB | Pre-ship `\d ai_tool_audit` / `\d forecast_computations` |
| CASCADE over-deletes ML history | Acceptable — ML recomputable; document |
| BB false positive after fix | Month-bucket SQL; no threshold change |
| BD error shows empty | Guard with `!isError` |
| Deploy ordering | Migration before backend image |

### Decisions

| ID | Topic | Summary |
|----|-------|---------|
| **DEC-0105** | Audit CHECK | DROP+ADD both constraints; `forecast_bucket_assignment` + extended statuses |
| **DEC-0106** | FK retention | `ON DELETE CASCADE` + ml_enhanced-before-baseline prune order |

Full records: `decisions/DEC-0105.md`, `decisions/DEC-0106.md`

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **AY**, **AZ** | AY1 | Audit rows post-sync; no CHECK WARN |
| **BA** | BA1, BA2, T1 | Recompute completes; meta fresh |
| **BB** | V1 (probe) | Month-bucket SQL + ML meta honest |
| **BC** | V1 (re-smoke) | Planning Compare no stale badge |
| **BD** | BD1, V1 | No false empty on nav |

### Next phase

`/sprint-plan` — materialize `/quick` sprint (≤6 tasks); then `/plan-verify`.

---

