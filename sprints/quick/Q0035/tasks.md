# Tasks — Q0035 (BUG-0027)

**Bug:** BUG-0027  
**Task count:** 5 mandatory (5/12 under `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260622-bug0027-q0035`

## Architecture → sprint mapping

| Architecture ID | Disposition | Gate |
|-----------------|-------------|------|
| **E1** | Task **E1** | GATE-ERROR-1, GATE-MESSAGE-1 |
| **E2** | Task **E2** | GATE-ERROR-1 |
| **T1** | Task **T1** | GATE-TEST-1 |
| **G1** | Task **G1** | — |
| **V1** | Task **V1** | — |
| Preflight probe | **Out of scope** (GATE-PREFLIGHT-1 deferred) | — |

## Execute order

```
E1
  → E2
  → T1
  → G1
  → operator: BACKEND_DEPLOY
  → V1 operator smoke
```

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **CB** | V1 | PAT regen + env update + container recreate → sync succeeds; no 401 in error_message |
| **CC** | E1, E2, T1, G1 | `FireflyError::Unauthorized` + wiremock test; display message on 401 |
| **CD** | V1 (with T1 coverage) | ≥3 scheduled syncs succeed post-regen |

| ID | Title | Est. | Status | Acceptance | Gate |
|----|-------|------|--------|------------|------|
| E1 | `FireflyError::Unauthorized` variant | 0.5h | open | **CC** | GATE-ERROR-1, GATE-MESSAGE-1 |
| E2 | Match 401 → `Unauthorized` | 0.5h | open | **CC** | GATE-ERROR-1 |
| T1 | wiremock 401 integration test | 1.5h | open | **CC**, **CD** | GATE-TEST-1 |
| G1 | Regression gates | 0.5h | open | all | — |
| V1 | Operator smoke: PAT regen + ≥3 syncs | 1h | open | **CB**, **CD** | — |

---

## E1 — `FireflyError::Unauthorized` variant

**Status:** open  
**Depends on:** —  
**Estimate:** 0.5h  
**Complexity:** Low (additive enum variant)  
**Risk:** Low (no existing behavior changes)  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0027 **CC** — **GATE-ERROR-1**, **GATE-MESSAGE-1**

### Description

Add `Unauthorized` unit variant to `FireflyError` enum in `backend/src/firefly/mod.rs` L20-37:

```rust
// L20-37 FireflyError enum
pub enum FireflyError {
    // ... existing variants ...
    PersonalAccessTokenMissing(String),
    Http(reqwest::Error),
    UnexpectedStatus(StatusCode),
    Unauthorized,  // NEW
}
```

Implement `Display` for `Unauthorized`:

```rust
FireflyError::Unauthorized => write!(f, "firefly_personal_access_token invalid or expired — regenerate in Firefly profile → API tokens → update FIREFLY_PERSONAL_ACCESS_TOKEN"),
```

**Files:** `backend/src/firefly/mod.rs` L20-37 (enum definition + Display impl)

### Done when

- [ ] `FireflyError::Unauthorized` unit variant added
- [ ] Display impl returns frozen message (GATE-MESSAGE-1)
- [ ] `cargo check --lib` compiles (test: compile check)

### Verification

`cargo check --lib` PASS; new variant visible in enum + Display.

---

## E2 — Match 401 → `Unauthorized` in request()

**Status:** open  
**Depends on:** E1  
**Estimate:** 0.5h  
**Complexity:** Low (additive match arm)  
**Risk:** Low (401 caught before fallthrough to `UnexpectedStatus`)  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0027 **CC** — **GATE-ERROR-1**

### Description

Add 401 match arm in `backend/src/firefly/mod.rs` L128-158 `request()` method:

```rust
// In request() method, status match block (L128-158)
match response.status() {
    // ... existing arms ...
    StatusCode::UNAUTHORIZED => return Err(FireflyError::Unauthorized),
    other => return Err(FireflyError::UnexpectedStatus(other)),
}
```

**Files:** `backend/src/firefly/mod.rs` L128-158 (request method status match)

### Done when

- [ ] 401 response returns `Err(FireflyError::Unauthorized)` instead of `UnexpectedStatus(401)`
- [ ] Other status codes unchanged (still `UnexpectedStatus`)
- [ ] `cargo check --lib` compiles

### Verification

`cargo check --lib` PASS; T1 integration test asserts behavior.

---

## T1 — wiremock 401 integration test

**Status:** open  
**Depends on:** E2  
**Estimate:** 1.5h  
**Complexity:** Medium (wiremock mock setup)  
**Risk:** Low (additive test — no production code changes)  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0027 **CC**, **CD** — **GATE-TEST-1**

### Description

Add wiremock-based test in `backend/tests/firefly_integration.rs`:

1. Start wiremock server
2. Mock `GET /api/v1/about` returning **401 Unauthorized** with `Accept: application/json` header match
3. Create FireflyClient pointing at wiremock with valid (but mock-rejected) PAT
4. Call any Firefly API method that invokes `request()`
5. Assert returned error is `FireflyError::Unauthorized` (not `UnexpectedStatus(401)`)
6. **Preserve** existing `PersonalAccessTokenMissing` test (verify not broken)

Follow the existing wiremock pattern in `firefly_integration.rs` (reuse test helpers, mock server setup).

**Files:** `backend/tests/firefly_integration.rs`

### Done when

- [ ] wiremock returns 401 for `/api/v1/about`
- [ ] Test asserts `FireflyError::Unauthorized` variant
- [ ] Existing `PersonalAccessTokenMissing` test still passes
- [ ] `cargo test --test firefly_integration` PASS

### Verification

New test in `firefly_integration.rs` runs green; old PAT-empty test still green.

---

## G1 — Regression gates

**Status:** open  
**Depends on:** E1, E2, T1  
**Estimate:** 0.5h  
**Complexity:** Low (run existing suites)  
**Risk:** Low  
**Acceptance hook:** BUG-0027 **CB**, **CC**, **CD**

### Description

Run and record in `sprints/quick/Q0035/progress.md`:

1. `cargo check --lib` → PASS
2. `cargo test --test firefly_integration` → PASS (includes T1 wiremock 401 + existing PAT-empty)
3. `cargo test` (sync tests + full cargo test suite) → PASS
4. `git diff --stat` blast radius matches frozen file list (only `backend/src/firefly/mod.rs`, `backend/tests/firefly_integration.rs`)

### Done when

- [ ] All checks PASS, recorded in progress.md

### Verification

Test output pasted in progress.md.

---

## V1 — Operator smoke: PAT regen + ≥3 scheduled syncs

**Status:** open  
**Depends on:** G1 + operator BACKEND_DEPLOY  
**Estimate:** 1h  
**Complexity:** Low (manual ops procedure)  
**Risk:** Low (idempotent PAT regen)  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0027 **CB**, **CC**, **CD**

### Description

Operator-run remediation (no automated test):

1. Regenerate PAT: Firefly profile → API tokens → create new token
2. Update `.env`: `FIREFLY_PERSONAL_ACCESS_TOKEN=<new-token>`
3. Recreate container: `docker compose up -d` (picks up new env)
4. Trigger sync: "Sync now" on `/sync` page → verify `state: completed`, non-zero entity counts, no `401` in `error_message`
5. Verify `GET /api/v1/sync/status` shows `state: completed`
6. Monitor **≥3 scheduled syncs** succeed (no silent 401 recurrence) → **CD**
7. Confirm `/sync` page displays clear diagnosis if 401 occurs again → **CC** ops check

**Operator gates:** **BACKEND_DEPLOY** (rebuild container with new error variant)

### Done when

- [ ] PAT regen + env update + container recreate completed
- [ ] Manual "Sync now" returns completed with entity counts
- [ ] ≥3 scheduled syncs succeed without 401
- [ ] `uat.md` populated with results

## UAT placeholders (DEC-0009)

- `sprints/quick/Q0035/uat.json` — sprint ID Q0035, empty steps array
- `sprints/quick/Q0035/uat.md` — target stories BUG-0027, acceptance CB/CC/CD (no results yet)
