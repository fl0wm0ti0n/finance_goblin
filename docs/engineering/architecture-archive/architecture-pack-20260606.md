# Architecture archive pack (2026-06-06)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=100`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 13
- First archived heading: `## BUG-0009 — Grafana empty panels & missing account value overview`
- Last archived heading: `## BUG-0009 — Grafana empty panels & missing account value overview`
- Verification tuple (mandatory):
  - archived_body_lines=219
  - preamble_lines=10
  - retained_body_lines=2802

---

## BUG-0009 — Grafana empty panels & missing account value overview

**Status:** architecture complete (2026-06-06)  
**Discovery:** `discovery-20260606-bug0009` in `handoffs/po_to_tl.md`  
**Research:** [R-0064](research.md#r-0064--bug-0009-grafana-panel-emptiness-vs-cross-account-overview-gap)  
**Decisions:** **DEC-0068** (Grafana analytics provisioning contract — variable default, portfolio overview, ML empty-state); extends **DEC-0009** (per-account scope), **DEC-0049** (ML default off), **DEC-0055** (Dashboard 5 ML panels), **DEC-0057** (unified analytics embed), **DEC-0065** (negative wealth visibility)  
**Sprint:** `/quick` **Q0016** recommended (6 tasks, under `SPRINT_MAX_TASKS=12`)  
**Acceptance:** `docs/product/acceptance.md` — BUG-0009 rows **(Y)**, **(Z)**  
**Related:** BUG-0004 DONE (datasource/UNION ruled out); BUG-0010 DONE (wealth totals); **US-0013** OPEN (ML enablement — out of scope for Y2); US-0011 DONE (embed routes)

### Symptom chain (frozen)

Operator on US-0010 external profile post-BUG-0004: 922+ transactions synced; Grafana analytics dashboards appear **empty** on cashflow/forecast at default load; portfolio account breakdown shows **1 of 3** accounts; no cross-account overview in analytics iframe.

| Sub | Root cause | Effect |
|-----|------------|--------|
| **Y1** | `$account_id` variable `ORDER BY name` → first row acct **116** (Cash wallet, zero forecast) | Cashflow + forecast-horizons panels flat zero at default load |
| **Y2** | ML panels hard-bind `model_kind='ml_enhanced'` — **0** computations on omniflow (DEC-0049) | Forecast-horizons ML section blank; reported as "empty panels" |
| **Y3** | Datasource/UNION regression | **Ruled out** — `POST /analytics/grafana/api/ds/query` **200** for all probed panels |
| **Z1** | Portfolio breakdown SQL: global `LIMIT 1` on cross-join with `jsonb_array_elements` | Latest snapshot truncated to **1 arbitrary account row** |
| **Z2** | No dedicated cross-account overview panel in analytics provisioning | AC Z unmet — React `/wealth` exists but outside Grafana iframe |

**Not transport/SQL regression:** Postgres datasource OK; portfolio total stat **-3395.75**; subscriptions **3/6**; UNION pie **200**.

`isolation_scope`: artifact + repo source + discovery public curl probes (R-0064); no host `.env` / secrets read.

### Fix contract (DEC-0068)

Provisioning-only fix — **no backend or React code required** unless sprint-plan adds optional Z3 docs copy.

```text
BUG-0009
├── Z1 — Portfolio breakdown SQL (P0)
│   └── portfolio.json: latest-snapshot subquery + LATERAL unnest; remove global LIMIT 1
├── Z2 — Cross-account overview (P0)
│   └── portfolio.json: stat row visibility + "All accounts" table (Z1 SQL); portfolio dashboard only
├── Y1 — $account_id default (P0)
│   └── cashflow.json + forecast-horizons.json: ORDER BY ABS(balance) DESC; omit saved current
├── Y2 — ML empty-state (P1)
│   └── forecast-horizons.json: text banner + noValue on ML panels; US-0013 boundary preserved
├── T1 — SQL + provisioning tests (P1)
│   └── fixtures: breakdown 3-row; variable query order; optional JSON snapshot
└── V1 — verify-work omniflow (P1)
    └── Y3/Z3: default-load cashflow non-empty; portfolio overview 3 rows; six routes smoke
```

**Out of scope:** US-0013 ML sidecar enablement; React `/forecast` API reorder (optional follow-up); seventh analytics landing dashboard; Grafana dynamic hide rules; backend changes.

**Execute order (frozen):** Z1 → Z2 → Y1 → Y2 → V1 (T1 parallel with Z1/Y1).

### Z1 — Portfolio breakdown SQL fix (frozen)

**Problem:** Panel id 5 (`portfolio.json` L142) applies `LIMIT 1` after cross-join — PostgreSQL returns one row from latest snapshot's account array, not the full list.

**Broken pattern (reject):**

```sql
SELECT elem->>'name' AS name, ...
FROM net_worth_snapshots, jsonb_array_elements(payload->'accounts') AS elem
ORDER BY snapshot_date DESC LIMIT 1
```

**Contract — subquery isolate snapshot, then unnest:**

```sql
SELECT
  elem->>'name' AS name,
  elem->>'account_role' AS role,
  elem->>'currency' AS currency,
  (elem->>'balance')::float AS balance
FROM (
  SELECT payload
  FROM net_worth_snapshots
  ORDER BY snapshot_date DESC
  LIMIT 1
) latest
CROSS JOIN LATERAL jsonb_array_elements(latest.payload->'accounts') AS elem
ORDER BY ABS((elem->>'balance')::float) DESC
```

| Rule | Detail |
|------|--------|
| Snapshot selection | Single latest row by `snapshot_date DESC` in subquery only |
| Unnest | `CROSS JOIN LATERAL jsonb_array_elements` on isolated `payload` |
| Sort | `ABS(balance) DESC` — aligns with overview prominence; negative Giro valid per DEC-0065 |
| Empty payload | Zero rows — valid post-sync empty-state |

**Alternatives rejected:** `DISTINCT ON (snapshot_date)` — unnecessary complexity; moving `LIMIT 1` into subquery only without LATERAL — still wrong if applied to outer join incorrectly.

**Files:** `grafana/provisioning/dashboards/analytics/portfolio.json` (panel id 5; Z2 reuses same SQL)

**Risks:** Mixed-currency without FX — native balances shown (existing `mixed_currency` warning stat handles expectation).

### Z2 — Cross-account overview panel (frozen)

**Problem:** AC Z requires operator cross-account value overview **in analytics** — Grafana summary panel/table or documented equivalent. React `/wealth` is supplementary only (Z3 docs), not sole fix.

**Contract — portfolio dashboard only:**

| Panel | Action |
|-------|--------|
| Stat row (existing) | Verify `total_eur`, `account_count`, mixed-currency warning visible above fold in kiosk embed (`DEC-0057`) |
| Table "All accounts (latest snapshot)" | Upgrade panel id 5 title/copy; use Z1 SQL; columns: name, role, currency, balance |
| Optional `% of Firefly subtotal` | SQL `pct_of_firefly` column when subtotal non-zero; show `—` when zero |
| Grid placement | Overview table immediately below stat row (`y` reposition); performance charts move down |
| Supplementary Z3 | Optional text panel: "Detailed wealth analysis → `/wealth`" — **not** AC Z substitute |

**Alternatives rejected:**

| Alternative | Why |
|-------------|-----|
| Overview on every dashboard | Provisioning duplication + drift (R-0064 §4) |
| Seventh "Overview" dashboard + sidebar | US-0011 scope expansion |
| React `/wealth` link only | Fails AC Z as primary fix |
| Grafana dynamic hide/show rules | Grafana 11 complexity; overkill for static provisioning |

**Files:** `grafana/provisioning/dashboards/analytics/portfolio.json`; optional `docs/user-guides/` Z3 copy only

**Risks:** Cashflow-first operators miss overview until sidebar navigation — mitigate via portfolio sidebar label hint in Z3 docs; not a code blocker.

### Y1 — `$account_id` variable default (frozen)

**Problem:** Both `cashflow.json` and `forecast-horizons.json` use `ORDER BY name` — Grafana picks first alphabetical row (acct **116**, zero forecast) when no `current` block is set.

**Contract — mirror balance sort (no backend change):**

```sql
SELECT a.firefly_id AS __value, a.name AS __text
FROM accounts a
WHERE a.type = 'asset'
ORDER BY ABS(COALESCE(a.balance, 0)) DESC, a.name ASC
```

| Rule | Detail |
|------|--------|
| Dashboards | `cashflow.json`, `forecast-horizons.json` — both account-scoped |
| `current` block | **Omit** from provisioning JSON — never bake operator-specific IDs |
| `refresh` | Keep `1` (on dashboard load) — already present |
| Tie-break | `name ASC` when ABS balances equal |
| All-zero deploy | Falls back to alphabetical — same as today; document in panel description |

**Alternatives rejected:**

| Alternative | Why |
|-------------|-----|
| First non-zero forecast subquery | Heavier; fails before first recompute (R-0064 §2) |
| Hardcoded `current` in JSON | Breaks other deployments |
| React localStorage → iframe `?var-account_id=` | US-0011 embed contract change — defer |
| Backend `/forecast/accounts` reorder alone | Does not fix Grafana embed |

**Files:** `grafana/provisioning/dashboards/analytics/cashflow.json`, `forecast-horizons.json` (templating `account_id`)

**Risks:** Multiple funded accounts — ABS picks largest magnitude; acceptable household MVP; manual Grafana save may bake `current` — runbook warning in V1.

### Y2 — ML panel empty-state (frozen)

**Problem:** `forecast-horizons` ML panels query `model_kind='ml_enhanced'` — zero rows on omniflow baseline-only profile (DEC-0049). Dashboard description alone insufficient — operators report blank charts as defect.

**Contract — banner + noValue (not hide, not US-0013):**

| Element | Action |
|---------|--------|
| Text panel | Row above ML section: *"ML forecast not enabled on this deployment. Baseline DEC-0007 forecast is authoritative. Enable via US-0013."* — aligned with DEC-0066 React copy |
| ML time-series panels | `fieldConfig.defaults.noValue` → `"ML unavailable"` |
| ML stat panels | Same `noValue` where applicable |
| `$forecast_variant` | **Unchanged** — default stays `baseline` |
| Dynamic hide rules | **Reject** — Grafana 11 show/hide complexity |

**Scope boundary:** US-0013 owns ML **enablement**; BUG-0009 closes **honest empty-state** only.

**Files:** `grafana/provisioning/dashboards/analytics/forecast-horizons.json`

**Risks:** ML charts still visually empty below banner — acceptable until US-0013; banner sets expectation.

### Task map (sprint-plan input)

| Order | Task | Layer | Acceptance | Est. |
|-------|------|-------|------------|------|
| 1 | **Z1** breakdown SQL subquery | grafana `portfolio.json` | **(Z)** | 1h |
| 2 | **Z2** overview table + grid layout | grafana `portfolio.json` | **(Z)** | 1.5h |
| 3 | **Y1** variable query ABS(balance) | grafana `cashflow.json`, `forecast-horizons.json` | **(Y)** | 1h |
| 4 | **Y2** ML banner + noValue | grafana `forecast-horizons.json` | **(Y)** | 1h |
| 5 | **T1** SQL fixtures + provisioning snapshot | tests | **(Y)(Z)** | 1.5h |
| 6 | **V1** verify-work omniflow | uat / operator smoke | **(Y)(Z)** | 1h |

**Total estimate:** ~7h (provisioning + tests; no backend deploy dependency beyond image rebuild for Grafana JSON).

**Deploy order:** Z1+Z2+Y1+Y2+T1 single PR → Grafana provisioning reload (container restart or provisioning poll) → V1 smoke on `financegnome.omniflow.cc`.

### Test strategy (frozen — maps acceptance Y/Z)

| Check | Type | Pass criteria |
|-------|------|---------------|
| Z1 — breakdown rows | SQL fixture | 3-account snapshot JSON → query returns **3 rows** (not 1) |
| Z2 — overview visible | Provisioning review | Portfolio table titled for all accounts; stat row present |
| Y1 — variable order | SQL fixture / snapshot | ABS(balance) DESC picks funded account (114) over zero wallet (116) |
| Y1 — default load | Operator smoke | `/analytics/cashflow` kiosk — no manual variable change → non-flat series |
| Y2 — ML banner | Provisioning review | Text panel present; ML panels have `noValue` |
| Y3 — datasource regression | Operator smoke | ds/query **200** for portfolio/subscriptions/budgets (unchanged) |
| Z3 — six routes | Operator smoke | All `/analytics/{slug}` routes render (US-0011 regression) |
| V1 — AC closure | verify-work | Rows **(Y)** and **(Z)** pass; `/wealth` documented supplementary |

### Decisions (BUG-0009)

| Topic | Resolution |
|-------|------------|
| Variable default | **DEC-0068** — `ORDER BY ABS(COALESCE(balance,0)) DESC, name`; omit `current` |
| Portfolio breakdown SQL | **DEC-0068** — latest-snapshot subquery + `LATERAL jsonb_array_elements` |
| Overview placement | **DEC-0068** — portfolio dashboard only; reject seventh dashboard |
| AC Z equivalence | Stat row + all-accounts table satisfies Z; `/wealth` supplementary (Z3) |
| ML empty-state | **DEC-0068** — banner + `noValue`; reject dynamic hide; US-0013 owns enablement |
| React API reorder | Optional follow-up — out of BUG-0009 execute |

### Next phase

**`/sprint-plan`** — quick sprint **Q0016**, 6 tasks (Z1, Z2, Y1, Y2, T1, V1).

---

