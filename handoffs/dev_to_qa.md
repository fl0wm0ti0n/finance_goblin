# Dev -> QA Handoff

## execute-20260607-q0017-bug0007-s-privacy — BUG-0007 Q0017 S privacy fix (verify-work loop)

**From:** Dev  
**To:** QA (`/qa`)  
**Date:** 2026-06-07  
**Bug:** BUG-0007  
**Sprint:** Q0017  
**Orchestrator:** `auto-20260607-bug0007-001`  
**Verify-work:** FAIL on row **(S)** — fix applied

### Summary

Verify-work loop fix for **(S)** — exempt subscription `display_name` and `merchant_names[]` from `PrivacyLayer` counterparty hashing in `get_subscriptions` tool output. Transaction payee/description redaction unchanged.

### Fix applied

| Finding | Fix | File |
|---------|-----|------|
| S-1/S-2 — LLM sees Counterparty-* instead of merchant names | Tool-aware walk preserves `display_name` / `merchant_names` for `get_subscriptions`; other strings still hashed | `backend/src/ai/privacy.rs` |

**Mechanism:** `walk_value_for_tool` sets `preserve_label_strings` when processing subscription label keys; strings under those keys skip counterparty hashing (IBAN redaction still applies).

### Tests run

| Check | Result |
|-------|--------|
| `cargo test --lib` | **PASS** (150/150) — includes 2 new privacy tests |
| `cargo test --test bug0007_ai_discovery` | **PASS** (8/8) |

**New unit tests:**
- `get_subscriptions_preserves_display_name_and_merchant_names`
- `get_subscriptions_still_redacts_other_long_strings`

### Deploy status

| Step | Result |
|------|--------|
| `docker build -f backend/Dockerfile -t flow-finance-ai:bug0007-s-fix .` | **PASS** |
| `docker compose --profile external build flow-finance-ai` | **BLOCKED** — `AUTHENTIK_SECRET_KEY` missing in env (compose validates oidc services) |

**Operator:** Re-deploy with omniflow compose command (same as verify-work) after merge; image builds cleanly via direct `docker build`.

### Acceptance impact

| Row | Prior | Expected after deploy + verify-work |
|-----|-------|-------------------------------------|
| **(S)** | FAIL | PASS — named merchants in LLM enumeration |
| **(T)** | PARTIAL | PARTIAL — undated 2023 window advisory (non-blocking) |
| **(U)** | PARTIAL | PASS expected when S unblocked |
| Regression | PASS | PASS — counterparty redaction preserved elsewhere |

### QA focus

1. Confirm privacy unit tests pass (`get_subscriptions_preserves_*`, `get_subscriptions_still_redacts_*`).
2. Confirm `cargo test --lib` 150/150 and `bug0007_ai_discovery` 8/8.
3. Confirm `get_transactions` payee/description still hashes to Counterparty-*.
4. **V1:** After operator BACKEND_DEPLOY, re-run verify-work S-1/S-2 probes per `sprints/quick/Q0017/uat.md`.

### Frozen boundaries (unchanged)

- Six-tool registry — no seventh tool
- `allow_raw_transactions=false` default
- BUG-0008 — additive AI JSON only
- RAG (V) — out of scope
- No frontend changes

### Artifacts

- `backend/src/ai/privacy.rs` — S fix
- `sprints/quick/Q0017/summary.md`
- `docs/engineering/state.md` — execute S-fix checkpoint
- `handoffs/resume_brief.md` — qa readiness

**Next phase:** `/qa` in new subagent/chat
