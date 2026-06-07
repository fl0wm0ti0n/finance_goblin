# Architecture archive pack (2026-06-06)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 13
- First archived heading: `## BUG-0006 — AI `get_transactions` empty despite synced mirror rows`
- Last archived heading: `## BUG-0007 — AI merchant/category discovery fails despite mirror data`
- Verification tuple (mandatory):
  - archived_body_lines=293
  - preamble_lines=10
  - retained_body_lines=2908

---

## BUG-0006 — AI `get_transactions` empty despite synced mirror rows

**Status:** architecture complete (2026-06-05)  
**Discovery:** `discovery-20260605-bug0006` in `handoffs/po_to_tl.md`  
**Research:** [R-0060](research.md#r-0060--ai-get_transactions-empty-aggregates-vs-mirror-sync)  
**Decisions:** **DEC-0059** (Firefly mirror amount sign normalization); extends **DEC-0002** (sync upsert backfill), **DEC-0032** (aggregate-only privacy)  
**Sprint:** `/quick` **Q0010** (recommended)  
**Acceptance:** `docs/product/acceptance.md` rows **P**, **Q**, **R**  
**Related:** BUG-0002–0005 OPEN — **do not merge**; separate deploy/verify tracks

### Symptom chain (frozen)

Operator: 922 transactions synced; AI Chat `get_transactions` (~23:30:13) → German “no expenses in categories / data unavailable”. Root cause is **three mirror ingest gaps** + **one aggregate contract gap** (not LLM-only):

| Sub | Gap | Effect on tool output |
|-----|-----|------------------------|
| **Q** | `category_id` never written | `by_category` rows with `category_name: null` |
| **Q2** | ISO datetime fails `%Y-%m-%d` parse → `date` NULL | Period filter returns **zero rows** |
| **Q3** | Positive Firefly amounts; SQL uses `amount < 0` for outflow | `total_outflow: 0`, `transaction_count > 0` |
| **R** | No period totals / `period_status` | LLM interprets empty arrays as “no data” |
| **P** | Downstream narrative | Fixed by Q/Q2/Q3/R + operator verify |

`isolation_scope`: artifact + repo source reads; no local `DATABASE_URL` probe; no host `.env` / `.env_prod` secrets read.

### Fix slices

```text
BUG-0006
├── Q — Mirror ingest (P0, ordered)
│   ├── Q1 — Extract `category_id` from first split; extend `upsert_transaction`
│   ├── Q2 — Parse ISO datetime dates → `NaiveDate` (date component only)
│   └── Q3 — Normalize amount sign per DEC-0059 before upsert
├── R — Aggregate contract (P0)
│   └── R1 — Period totals + `uncategorized_transaction_count` + `period_status`
└── P — Operator verify (P1)
    └── P1 — E2E on financegnome.omniflow.cc + SQL probe checklist
```

**Deploy order:** Q1 → Q2 → Q3 → R1 in one backend PR (ingest before aggregates); trigger **manual Firefly sync** (or wait for scheduled sync) to backfill existing ~922 rows via DEC-0002 upsert — **no SQL migration script**.

### Firefly ingest contract (frozen)

Single mirror row per Firefly journal (unchanged): **first split** in `attributes.transactions[]` drives scalar columns; full journal in `payload` JSONB.

| Mirror column | Firefly source | Contract |
|---------------|----------------|----------|
| `firefly_id` | top-level `id` | unchanged |
| `account_id` | first split `source_id` | unchanged |
| `date` | first split `date` (or attrs `date`) | **Q2:** accept `YYYY-MM-DD` **or** ISO-8601/RFC3339 datetime; persist **date component only** (`NaiveDate`). Parse order: strict date → RFC3339 → first-10-char prefix fallback. NULL only when all parses fail (log `warn`). |
| `category_id` | first split `category_id` (string) | **Q1:** extract and upsert; NULL when absent in payload |
| `amount` | first split `amount` + `type` | **Q3 / DEC-0059:** store **signed** EUR magnitude (see below) |
| `description` | first split `description` | unchanged |
| `payload` | full journal `item` | unchanged (read-only GET per DEC-0004) |

**Re-sync backfill (decision gate):** Rely on `ON CONFLICT (firefly_id) DO UPDATE` + DEC-0002 7-day overlap watermark — existing rows refresh on next sync. **Rejected:** one-off SQL backfill script (operator burden); changing aggregate SQL to read unsigned amounts from `payload` (duplicates logic across forecast/subscriptions/alerts).

**Files (ingest):** `backend/src/firefly/mod.rs`, `backend/src/db/mod.rs` (`upsert_transaction` adds `category_id` bind).

**Risks:** First-split-only model misses destination-leg category on some transfers — acceptable MVP; full journal still in `payload`. ISO parse edge cases (timezone midnight) — use date component of parsed instant only.

### Amount sign normalization — DEC-0059

Firefly split API stores **positive** `amount`; direction is split `type`. All mirror consumers (`TransactionsRepository`, subscriptions, forecast, alerts) treat **`amount < 0` as outflow**.

Normalize in `sync_transactions` **before** `upsert_transaction`:

| Split `type` | Stored `amount` |
|--------------|-----------------|
| `withdrawal` | `-abs(amount)` |
| `deposit` | `+abs(amount)` |
| `transfer` | `-abs(amount)` on ingested source leg (first split) |
| missing / unknown | `-abs(amount)` when raw `amount > 0`, else raw value; `tracing::debug` |

**Alternatives rejected:**

- *Negate withdrawal only, leave transfer/deposit positive* — leaves Q3 failure for mixed ledgers.
- *Account-role heuristic (source vs destination)* — heavier; first-split + `type` sufficient for household aggregates.
- *Rewrite all aggregate SQL to `ABS(amount)` + payload `type`* — cross-cutting; violates single normalization point.

**Risks:** Transfer sign on first leg may mis-classify rare journals — monitor via `period_status`; payload retains audit trail. Subscription/forecast recompute after backfill may shift — expected correction.

### `TransactionAggregates` API contract — R1 (frozen)

Extend `backend/src/transactions/types.rs` and assemble in `TransactionsService::aggregates` (plus `GetTransactionsTool` passthrough). **Privacy:** new fields are numeric aggregates only — **no change** to `PrivacyLayer` (`DEC-0032`); `raw_rows` still gated by `allow_raw_transactions`.

**New top-level fields** (always present in tool JSON):

| Field | Type | Semantics |
|-------|------|-----------|
| `total_transaction_count` | `i64` | `COUNT(*)` where `date` in `[period_start, period_end]` inclusive |
| `total_outflow` | `f64` | `SUM(ABS(amount))` where `amount < 0` |
| `total_inflow` | `f64` | `SUM(amount)` where `amount > 0` |
| `uncategorized_transaction_count` | `i64` | rows in period with `category_id IS NULL` |
| `period_status` | enum (snake_case JSON) | LLM empty-state hint (see below) |

**`period_status` enum** — mutually exclusive, evaluated in **priority order**:

| Value | Condition |
|-------|-----------|
| `no_rows_in_period` | `total_transaction_count == 0` |
| `rows_zero_outflow` | count > 0 and `total_outflow == 0.0` |
| `rows_uncategorized` | count > 0 and `uncategorized_transaction_count == total_transaction_count` |
| `rows_with_outflow` | count > 0 and `total_outflow > 0.0` |

**`by_category` presentation:** Keep SQL `GROUP BY category_id`. Rows with `category_id IS NULL` map to `category_name: "Uncategorized"` in service assembly (not DB join). **Rejected:** top-level count only without NULL bucket in `by_category` — LLM needs labeled bucket under aggregate-only mode.

**Existing fields unchanged:** `period_start`, `period_end`, `group_by`, `by_category`, `by_month`, `raw_rows` (null when `allow_raw_transactions=false`).

**Repository:** add `period_summary(start, end) -> (count, outflow, inflow, uncategorized_count)`; reuse outflow/inflow CASE expressions from `aggregates_by_category` for consistency.

**Files (R1):** `backend/src/transactions/types.rs`, `repository.rs`, `service.rs`, `backend/src/ai/tools/transactions.rs` (schema description optional).

**Risks:** LLM may still misread if `period_status` ignored — mitigated by explicit totals; tool description update optional in execute. Floating-point totals — use same `float8` SUM semantics as existing aggregates.

### Task map (Q0010)

| Order | Task | Layer | Acceptance |
|-------|------|-------|------------|
| 1 | **Q1** category sync | backend ingest | **Q** |
| 2 | **Q2** ISO date parse | backend ingest | **Q** (dates in period) |
| 3 | **Q3** amount sign (DEC-0059) | backend ingest | **Q** + outflow sums |
| 4 | **R1** aggregate contract | backend transactions + AI tool | **R** |
| 5 | **P1** operator E2E + SQL probe | verify-work | **P** |

**Count:** 5 tasks (≤ `SPRINT_MAX_TASKS` 12) → **`/quick` Q0010**; no split.

### Test strategy

| Check | Type | Pass criteria |
|-------|------|---------------|
| Q1 | Unit | Fixture split JSON → `category_id` persisted on upsert |
| Q2 | Unit | ISO datetime string → non-NULL `NaiveDate` |
| Q3 | Unit | `withdrawal`/`deposit`/`transfer` → expected sign |
| R1 | Integration | Fixture mirror rows → JSON with totals + correct `period_status` |
| Privacy | Unit | `allow_raw_transactions=false` → new fields present; `raw_rows` absent; PrivacyLayer unchanged |
| P1 | Operator | Chat spending question uses non-empty aggregates; SQL probe matches tool totals |

**Post-deploy operator step:** Manual Firefly sync after deploy to backfill ~922 rows before P1 verify.

### Decisions (BUG-0006)

| Topic | Resolution |
|-------|------------|
| Amount sign | **DEC-0059** — normalize at ingest from split `type` |
| Re-sync backfill | **Upsert on next sync** (DEC-0002) — no migration script |
| Uncategorized bucket | **Top-level count + `by_category` "Uncategorized" label** |
| Privacy | **Aggregate-only** — DEC-0032 unchanged |
| Merge with BUG-0004 | **Rejected** |

### Next phase

**`/sprint-plan` Q0010** — materialize `sprints/quick/Q0010/task.json` from task table; then `/plan-verify` → `/execute`.

---

## BUG-0007 — AI merchant/category discovery fails despite mirror data

**Status:** architecture complete (2026-06-07)  
**Discovery:** `discovery-20260607-bug0007` in `handoffs/po_to_tl.md`  
**Research:** [R-0065](research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag)  
**Decision:** **DEC-0069** (A′ + E + F bundle)  
**Sprint:** `/quick` **Q0017** (recommended)  
**Acceptance:** `docs/product/acceptance.md` rows **S**, **T**, **U** (note **V** — RAG deferred, not acceptance gate)  
**Related:** BUG-0006 DONE (mirror ingest); BUG-0008 OPEN — coordinate only; US-0015 OPEN — bucket mapping out of scope

### Symptom chain (frozen)

Operator on US-0010 external profile: 922 transactions synced (2025-06-05…2026-05-22), 75 categories, 12 subscription patterns with named merchants. AI Chat fails subscription enumeration (**S**), merchant/category queries (**T**), and cross-signal fusion (**U**).

| Sub | Verdict | Root cause |
|-----|---------|------------|
| **S** | CONFIRMED | Tool returns `display_name`; LLM does not enumerate; follow-up passes `Counterparty-*` as enum filters |
| **T** | SPLIT | **T-a:** Amazon Jan–Oct 2023 — true empty (mirror starts 2025-06-05). **T-b:** Keywords passed as `category_id`; no name-search dimension |
| **U** | CONFIRMED | Aggregate-only + redaction + prompt bias — no fusion path without user-supplied merchant names |
| **V** | NOTE | No RAG layer — defer; not acceptance gate |

`isolation_scope`: artifact + repo source reads; live omniflow probes (discovery); no host `.env` / `.env_prod` secrets read.

### Fix slices (DEC-0069)

```text
BUG-0007
├── A′ — Category search on get_transactions (P0)
│   ├── A1 — Repository: ILIKE category name resolve + mirror MIN/MAX bounds
│   └── A2 — Service + tool schema/response extensions
├── F — get_subscriptions schema (P0)
│   └── F1 — kind enum, merchant_names[], patterns_count, Counterparty guard
├── E — Orchestrator + audit (P0)
│   ├── E1 — SYSTEM_PROMPT rules (enumerate, category_search, bounds, enums)
│   └── E2 — audit.result_rows population
└── V1 — Operator verify (P1)
    └── V1 — AI Chat probes S/T/U on financegnome.omniflow.cc
```

**Deploy order:** (A1 → A2 → F1 → E1 → E2) single backend PR → deploy → V1 verify. **No** Firefly re-sync required (intelligence-layer only).

### A′ — `get_transactions.category_search` contract (frozen)

Extends BUG-0006 `TransactionAggregates` — see **DEC-0069 § A′** for full field table.

| Concern | Contract |
|---------|----------|
| **Param name** | `category_search` (not `category_name_query`) |
| **Resolution** | `categories.name ILIKE '%keyword%'`; cap **10** matches; union-filter aggregates |
| **Precedence** | `category_search` wins when both `category_id` + search supplied |
| **Mirror bounds** | `mirror_date_bounds { min, max }` on **every** response — global mirror date range |
| **Empty keyword match** | `category_matches: []`, `search_attempted: true`, existing `period_status` |
| **Privacy** | DEC-0032 unchanged; no `raw_rows`; no payee/description search |

**Omniflow proof (frozen):** `category_search: "amazon"` → id **47** (`Shopping - Amazon`, 1079.35 €); `"strom"` → id **146** (`Wohnen - Stromkosten`, 465.53 €).

**Alternatives rejected:**

| Alternative | Why |
|-------------|-----|
| Seventh `get_categories` tool | Six-tool AC footer |
| Payee aggregates (`group_by: merchant`) | Counterparty redaction → unreadable labels (R-0065 §6) |
| RAG / vector search | No infra; epic scope (note V) |
| `allow_raw_transactions` default flip | Privacy regression; supplementary only |

**Files:** `backend/src/transactions/{repository,service,types}.rs`, `backend/src/ai/tools/transactions.rs`.

### F — `get_subscriptions` contract (frozen)

| Concern | Contract |
|---------|----------|
| **`kind` enum** | `subscription` \| `standing_order` in OpenAI schema |
| **Enum guard** | Reject `Counterparty-*` prefix in `status`/`kind` → InvalidArgs |
| **Response** | Add `patterns_count`, `merchant_names[]` (deduped display_name order) |
| **`patterns[]`** | Unchanged per-field shape |
| **REST API** | **No change** — enrichment in AI tool wrapper only (BUG-0008 isolation) |

**Files:** `backend/src/ai/tools/subscriptions.rs` (primary); `SubscriptionService::list_patterns` behavior unchanged.

### E — Orchestrator + audit contract (frozen)

| Concern | Contract |
|---------|----------|
| **SYSTEM_PROMPT** | Four rules: enumerate subscriptions; use category_search for keywords; cite mirror_date_bounds on empty period; never Counterparty-* enums |
| **audit.result_rows** | `get_transactions`: bucket count or total_transaction_count; `get_subscriptions`: patterns_count |
| **Schema descriptions** | Enrich `category_id` + `category_search` parameter docs |
| **Local providers** | No `tool_choice: required` (DEC-0045) |

**Files:** `backend/src/ai/orchestrator.rs`.

### BUG-0008 coordination (note only)

Shared `SubscriptionService` — BUG-0007 execute is **additive AI JSON only**. Do **not** change alert unread count, `/subscriptions` list filters, or detection thresholds. See DEC-0069 coordination table.

### Task map (Q0017)

| Order | Task | Layer | Est. | Acceptance |
|-------|------|-------|------|------------|
| 1 | **A1** category search SQL + mirror bounds | backend transactions | 3h | **T**, **U** |
| 2 | **A2** tool schema + response assembly | backend AI tool | 2h | **T**, **U** |
| 3 | **F1** subscriptions schema + response + guard | backend AI tool | 2h | **S** |
| 4 | **E1** SYSTEM_PROMPT + audit result_rows | backend orchestrator | 2h | **S**, **T**, **U** |
| 5 | **E2** parameter schema descriptions | backend AI tools | 0.5h | **T**, **S** |
| 6 | **T1** unit/integration tests | backend tests | 3h | regression |
| 7 | **V1** operator AI Chat verify | verify-work | 1h | **S**, **T**, **U** |

**Count:** 7 tasks (≤ `SPRINT_MAX_TASKS` 12) → **`/quick` Q0017**; no split.  
**Total estimate:** ~13.5h (dev ~12.5h + operator V1 ~1h).

### Test strategy

| Check | Type | Pass criteria |
|-------|------|---------------|
| A1 | Unit | ILIKE resolves Strom/Amazon fixture categories; bounds query returns min/max |
| A2 | Integration | Tool JSON includes `category_matches`, `mirror_date_bounds`, `search_attempted` |
| F1 | Unit | Counterparty-* args rejected; `merchant_names` deduped; kind enum in schema |
| E1 | Unit | Audit insert receives non-NULL `result_rows` for both tools |
| Privacy | Unit | Six-tool registry count; `allow_raw_transactions=false` → no `raw_rows` |
| V1 | Operator | Chat lists subscription names; Strom/Amazon amounts; 2023 Amazon cites bounds |

### Decisions (BUG-0007)

| Topic | Resolution |
|-------|------------|
| Category resolution | **DEC-0069 A′** — `category_search` on `get_transactions` |
| Mirror empty-state | **DEC-0069** — `mirror_date_bounds` on every tool response |
| Subscription enumeration | **DEC-0069 F** — schema + `merchant_names` |
| Orchestrator / audit | **DEC-0069 E** — prompt rules + `result_rows` |
| Six-tool registry | **Preserved** — no seventh tool |
| RAG (V) | **Deferred** — document only |
| BUG-0008 | **Coordinate** — additive AI JSON only |

### Next phase

**`/sprint-plan` Q0017** — materialize `sprints/quick/Q0017/task.json` from task table; then `/plan-verify` → `/execute`.

---

