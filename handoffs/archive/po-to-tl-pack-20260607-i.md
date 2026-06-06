# PO→TL Archive Pack — BUG-0007 discovery + research

**Archived:** 2026-06-07T24:00:00Z (refresh-context post-BUG-0007)
**Source:** `handoffs/po_to_tl.md` compaction
**Closed:** BUG-0007 / Q0017 — verify-work PASS + release PASS (2026-06-07)
**Orchestrator:** `auto-20260607-bug0007-001`

Non-destructive append-only archive.

---

## research-20260607-bug0007 — BUG-0007 AI merchant/category discovery

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-07  
**Work item:** BUG-0007 (defect)  
**Orchestrator:** `auto-20260607-bug0007-001`  
**Next phase:** `/architecture`

### Summary

Research expanded **[R-0065](docs/engineering/research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag)** with fix-option matrix, risks, dependencies, and BUG-0008 isolation. Web refs: OpenAI function-calling (enum constraints, opaque-ID companion resolution), privacy aggregate-first analogues. **Recommended fix path: A′ + E + F** — extend `get_transactions` with server-side `category_search` (not a seventh tool), enrich `get_subscriptions` schema, orchestrator prompt + audit `result_rows`. **Defer:** payee aggregates (B — useless under redaction), RAG (D), raw opt-in (C — supplementary only). Six-tool registry + `allow_raw_transactions=false` default preserved.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **Category resolution (T/U)** | [R-0065 §4 A′](docs/engineering/research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag) | **`category_search` param on `get_transactions`** — ILIKE on synced `categories.name`; cap matches; optional mirror date bounds in response |
| **Separate catalog tool (A)** | Six-tool AC footer | **Reject MVP** — violates acceptance unless DEC waiver; A′ achieves same outcome |
| **Subscription enumeration (S)** | [R-0065 §4 F](docs/engineering/research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag) | Tighten `kind` enum; prompt + schema instruct listing all `display_name`; no `payee_key` required |
| **Orchestrator / audit (E)** | [R-0065 §5](docs/engineering/research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag) | Extended system prompt; populate `audit.result_rows`; block `Counterparty-*` enum misuse |
| **Payee aggregates (B)** | Redaction hashes payee → `Counterparty-*` | **Defer** — Amazon signal is category name on omniflow; not acceptance blocker |
| **RAG (V/D)** | No vector layer | **Defer** — document in architecture; not acceptance gate |
| **BUG-0008** | Shared `SubscriptionService` | Additive AI JSON only — no alert/list/detection changes |

### Recommended fix path

**Primary bundle (architecture baseline):** **A′ + E + F**

1. **A′** — `get_transactions.category_search` resolves Strom→146, Amazon→47 server-side within six tools.
2. **F** — `get_subscriptions` schema enrichment (`kind` enum, enumeration guidance).
3. **E** — orchestrator prompt rules + audit row counts + mirror-bound empty states for T-a (2023 Amazon).

**Cross-signal fusion (U):** multi-tool path via category search + subscription `display_name` + category aggregates — no raw descriptions required under DEC-0032 defaults.

### Risks surfaced (carry to architecture)

1. **Broad `category_search` matches** — cap results; document precedence vs `category_id`.
2. **Local model prompt adherence** — mitigated by schema descriptions + existing local nudge (R-0041).
3. **Shared subscription service** — BUG-0007 JSON additions must not alter BUG-0008 alert/list semantics.
4. **Six-tool waiver temptation** — A′ preferred over seventh tool to preserve AC footer.

### Recommended next steps

1. `/architecture` — DEC for A′ param contract, E prompt/audit, F subscription schema; BUG-0008 coordinate table
2. `/sprint-plan` — quick task after architecture (US-0006-scoped execute expected)

---

## discovery-20260607-bug0007 — BUG-0007 AI merchant/category discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-07  
**Work item:** BUG-0007 (defect)  
**Orchestrator:** `auto-20260607-bug0007-001`  
**Next phase:** `/research` (seeded — [R-0065](docs/engineering/research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag)) → `/architecture`

### Summary

Discovery probed **live omniflow** (`financegnome.omniflow.cc`, US-0010 external profile) and **US-0006 tool code paths** for sub-defects **S/T/U** (+ note **V**). Mirror holds **922 transactions** (dates **2025-06-05…2026-05-22**), **75 categories**, and **12 subscription patterns** with merchant `display_name` values. AI Chat failures are **primarily tool-contract + orchestration gaps**, not missing sync data (post-BUG-0006). No `.env` / `.env_prod` secrets read.

### Sub-defect verdicts

| Sub | Verdict | Root cause (evidence) |
|-----|---------|----------------------|
| **S** | **CONFIRMED** | `get_subscriptions` returns named patterns (Netflix, YouTube, Apple, Cursor, …); audit shows **ok** `{}` calls but LLM still generic-lists / "cannot retrieve"; follow-up calls pass `Counterparty-*` as `kind`/`status` enums |
| **T** | **SPLIT** | **T-a:** Amazon Jan–Oct 2023 → **0 mirror rows** (true empty period). **T-b:** Strom/Amazon in mirror → categories **146/47/18** hold spend; LLM passes `category_id: "Strom"` / `"amazon"` (keyword ≠ Firefly id) |
| **U** | **CONFIRMED** | No fusion path: aggregate-only `get_transactions`, `redact_counterparties`, system prompt "prefer aggregates", no category catalog tool |
| **V** | **NOTE** | No RAG in codebase; [R-0065](docs/engineering/research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag) documents tool vs RAG tradeoff — not acceptance gate |

### Live runtime proof (names-only)

| Probe | Result |
|-------|--------|
| `GET /health` | 200 `{"status":"ok"}` |
| `GET /api/v1/sync/entities` | transactions **922**, categories **75** |
| `GET /api/v1/settings` | `allow_raw_transactions=false`, `redact_counterparties=true` |
| Grafana SQL date range | `MIN(date)=2025-06-05`, `MAX(date)=2026-05-22` |
| Category totals | Amazon **1079.35 €** (28 tx), Stromkosten **465.53 €** (4 tx), Streaming **350.51 €** (20 tx) |
| `GET /api/v1/subscriptions` | 12 patterns (3 confirmed, 6 pending, 3 rejected) |
| `GET /api/v1/ai/audit` | 2026-06-05 session: mis-parameterized `category_id` + `Counterparty-*` subscription filters |

### Code-path evidence

| Path | Finding |
|------|---------|
| `backend/src/ai/tools/transactions.rs` | Only period + `category_id` (Firefly id) + `group_by`; no merchant/description search |
| `backend/src/ai/tools/subscriptions.rs` | Returns patterns via `SubscriptionService::list_patterns` — omits `payee_key` vs REST |
| `backend/src/subscriptions/service.rs` | AI JSON: `display_name`, `status`, `kind`, amounts — sufficient for S if LLM uses payload |
| `backend/src/ai/privacy.rs` | `allow_raw_transactions=false` blocks `raw_rows`; counterparty hashing on description/payee |
| `backend/src/ai/orchestrator.rs` | `SYSTEM_PROMPT` biases aggregates; audit stores args only (`result_rows` always None) |

### Discovery decomposition evidence

| Evaluator | Result |
|-----------|--------|
| Feature/workflow count | S confirmed; T split (period-empty vs tool-contract); U confirmed; V note |
| Cross-cutting | US-0006 orchestrator, two tools, privacy layer, subscription detection |
| Acceptance breadth | Rows S/T/U unchanged — T operator repro partially explained by mirror date range |
| Risk | Fix likely spans tool schema + orchestrator + optional new catalog tool — architecture required |

### RAG vs tools (note V — carry to research/architecture)

- **No RAG/vector layer** exists today.
- [R-0065](docs/engineering/research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag) recommends lean path: **category catalog tool + orchestrator/audit improvements**; evaluate payee aggregates; defer RAG; `allow_raw_transactions` supplementary only with operator opt-in.
- **No architecture prescription at discovery.**

### Coordinate (do not merge)

- **BUG-0008** — subscription alert/list mismatch (UI surface).
- **BUG-0006** — DONE; aggregates/category ingest fixed — this bug is intelligence-layer.
- **US-0015** — forecast bucket mapping epic (different surface).

### Triad check (discovery phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#BUG-0007` | Discovery notes + verdicts | pass |
| `docs/product/acceptance.md` BUG-0007 | Rows S/T/U unchanged | pass |
| `docs/engineering/research.md` | [R-0065](docs/engineering/research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag) appended | pass |
| `handoffs/intake_evidence/intake-20260605-ai-merchant-category-discovery.json` | Intake evidence aligned | pass |

### Recommended next steps

1. `/research` — confirm [R-0065](docs/engineering/research.md#r-0065--bug-0007-ai-merchant-category-discovery-tool-contracts-vs-rag) options (category catalog tool, payee aggregates, prompt/audit) — no host secrets
2. `/architecture` — DEC for tool contract changes, privacy boundary, orchestrator prompt; coordinate BUG-0008 only
3. `/sprint-plan` — quick task after architecture (expect US-0006-scoped execute)

