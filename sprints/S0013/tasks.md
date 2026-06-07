# Tasks — Sprint S0013

**Story:** US-0016  
**Task count:** 7 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Acceptance refs |
|----|-------|--------|-----------------|
| T-0137 | Root README split layout skeleton | done | AC-1, AC-2 |
| T-0138 | Purpose + Product status seed | done | AC-1, AC-5 |
| T-0139 | Quickstart, Examples, Limitations content | done | AC-1 |
| T-0140 | Related documentation + compose commands | done | AC-3 |
| T-0141 | Doc validator CI gate | done | AC-4 |
| T-0142 | Runbook README maintenance hooks | done | AC-5 |
| T-0143 | Developer shard maintenance pointer | done | AC-5 |

---

## T-0137 — Root README split layout skeleton

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0070, DEC-0059 (doc profile)  
**Architecture slice:** R1

### Description

Create root `README.md` with frozen DEC-0059 split layout for active profile (`DOC_AUDIENCE_PROFILE=both`, `DOC_DETAIL_LEVEL=balanced`):

| Element | Requirement |
|---------|-------------|
| User H2s | Exact titles: `Purpose`, `Quickstart`, `Examples`, `Limitations`, `Related documentation` |
| Contributing | Single `## Contributing` → link [`docs/developer/README.md`](docs/developer/README.md) |
| Forbidden | Zero `DEV_*` H2 titles in root (`doc_profile_lib.dev_h2_forbidden_in_root`) |

Establish section scaffolding only — substantive content lands in T-0138–T-0140. Each user H2 must have at least one non-empty paragraph (no `TODO` / `TBD` stubs that fail validator).

### Done when

- [ ] `README.md` exists at repo root
- [ ] All five required user H2s present with exact titles
- [ ] `## Contributing` points to `docs/developer/README.md`
- [ ] No `DEV_*` H2 titles in root
- [ ] File is valid Markdown; no secrets or host-specific credentials

---

## T-0138 — Purpose + Product status seed

**Status:** open  
**Depends on:** T-0137  
**Decisions:** DEC-0070  
**Architecture slice:** R2  
**Research:** R-0066, R-0067 §2

### Description

Populate `## Purpose` with Flow Finance AI value proposition (personal finance analytics, Firefly sync, forecasting, Grafana analytics, AI assistant — sourced from product vision/backlog, not generic stubs).

Immediately under Purpose, add **`### Product status`** per DEC-0070:

| Contract | Value |
|----------|-------|
| Format | `{US-xxxx\|BUG-xxxx} — {one-line outcome}` reverse-chronological |
| Cap | **8** bullets maximum (drop oldest) |
| History | Link `docs/product/backlog.md` for full history |
| Seed | Include recent closed items (e.g. Q0017/BUG-0007, Q0016/BUG-0009) as initial bullets |

**Forbidden:** dedicated `## Product status` H2; acceptance-table dumps; secrets.

### Done when

- [ ] Purpose section has Flow Finance AI-specific prose (not placeholder)
- [ ] `### Product status` immediately under `## Purpose`
- [ ] At least one seeded bullet in `{id} — outcome` format
- [ ] Backlog link present for full history
- [ ] Bullet count ≤ 8

---

## T-0139 — Quickstart, Examples, Limitations content

**Status:** open  
**Depends on:** T-0137  
**Decisions:** DEC-0070  
**Architecture slice:** R1  
**Research:** R-0066

### Description

Populate three user H2 sections with Flow Finance AI-specific, copy-paste-friendly content:

| Section | Content source |
|---------|----------------|
| **Quickstart** | Compose profiles from `.env.example` comments: minimal, bundled-firefly, external omniflow; env copy steps; no literal secrets |
| **Examples** | Sync + analytics routes; representative operator flows |
| **Limitations** | Known sharp edges; unsupported envs; honest ML/forecast caveats where applicable |

Reference `.env.example` and `docs/engineering/runbook.md` for commands — do not duplicate full runbook prose.

### Done when

- [ ] Quickstart documents all three compose entry profiles with commands
- [ ] Examples section has actionable copy-paste examples
- [ ] Limitations section lists real product constraints (non-stub)
- [ ] No secrets, tokens, or host-specific credentials in README

---

## T-0140 — Related documentation + compose commands

**Status:** open  
**Depends on:** T-0137  
**Decisions:** DEC-0070  
**Architecture slice:** R3  
**Research:** R-0066

### Description

Populate `## Related documentation` with cross-links:

| Link | Required |
|------|----------|
| `docs/user-guides/` | Yes (`USER_GUIDE_MODE=1`) |
| `docs/engineering/runbook.md` | Yes |
| `docs/engineering/architecture.md` or decisions index | Recommended |
| `docs/engineering/spec-pack/` paths | When `SPEC_PACK_MODE=1` |

Reiterate minimal / bundled-firefly / external omniflow compose entry commands (may reference Quickstart to avoid duplication, but AC-3 requires compose commands documented in Related documentation or clearly cross-linked).

### Done when

- [ ] `docs/user-guides/` linked
- [ ] `docs/engineering/runbook.md` linked
- [ ] Engineering/spec paths mentioned when `SPEC_PACK_MODE=1`
- [ ] All three compose profiles documented or clearly cross-referenced with commands
- [ ] Links use repo-relative paths

---

## T-0141 — Doc validator CI gate

**Status:** open  
**Depends on:** T-0137, T-0138, T-0139, T-0140  
**Decisions:** DEC-0070  
**Architecture slice:** R4  
**Research:** R-0067 §1

### Description

Wire `validate_doc_profile.py --no-template-parity` into the CI/local test path while `template/` is absent:

```bash
python scripts/validate_doc_profile.py --repo . --no-template-parity
```

| Touch point | Change |
|-------------|--------|
| `tests/run-tests.sh` | Add doc profile validation step before "All tests passed" |
| `docs/engineering/runbook.md` `TEST_COMMAND` | Ensure doc gate runs as part of canonical test command (if not already) |

Non-zero exit must fail the test script (fail closed). Do **not** add partial `template/README.md` stub.

Local verification: command exits **0** with completed README content.

### Done when

- [ ] `tests/run-tests.sh` invokes `validate_doc_profile.py --repo . --no-template-parity`
- [ ] Validator exits **0** locally after README tasks complete
- [ ] `TEST_COMMAND` in runbook includes doc gate (directly or via `run-tests.sh`)
- [ ] No `--no-template-parity` removal (flip gate deferred until `template/` tree)

---

## T-0142 — Runbook README maintenance hooks

**Status:** open  
**Depends on:** — (may parallelize with README content tasks)  
**Decisions:** DEC-0070  
**Architecture slice:** R5  
**Research:** R-0067 §3

### Description

Add subsection **`README maintenance (US-0016)`** under runbook § documentation profile validation. Embed frozen phase-boundary hooks:

#### Release (`/release`)

After backlog reconciliation (≈ step 10), before runbook readiness (≈ step 14):

1. For each US/BUG in target sprint → DONE/CLOSED, append one Product status bullet.
2. Trim to 8 most recent entries.
3. Run `python scripts/validate_doc_profile.py --repo . --no-template-parity` — non-zero → fail closed.

#### Refresh-context (`/refresh-context`)

After backlog status reconciliation:

1. Verify Product status includes closed id(s) since prior refresh; update if missing.
2. If README or doc-profile surfaces touched, run validator with `--no-template-parity`.

#### Template flip gate

Document: drop `--no-template-parity` only when full `template/README.md` + `template/docs/developer/README.md` land in the same change set.

Document **both** validator commands (with and without `--no-template-parity`) and when to use each.

### Done when

- [ ] `README maintenance (US-0016)` subsection exists under § documentation profile validation
- [ ] Release hook checklist present with exact validator command
- [ ] Refresh-context hook checklist present
- [ ] Template flip gate documented
- [ ] Both validator command variants documented with usage guidance

---

## T-0143 — Developer shard maintenance pointer

**Status:** open  
**Depends on:** T-0142  
**Decisions:** DEC-0070  
**Architecture slice:** R6

### Description

Add one sentence to `docs/developer/README.md` § Workflow or § Quality gates:

> After `/release` or `/refresh-context` closes backlog items, curators/release agents update root README **Product status** per runbook § README maintenance (US-0016); contributors run `validate_doc_profile` when editing README surfaces.

Do not duplicate full maintenance checklist — pointer only.

### Done when

- [ ] One-sentence pointer added to Workflow or Quality gates section
- [ ] Pointer references runbook § README maintenance (US-0016)
- [ ] No structural changes to DEV_* H2 layout in developer shard

---

## Execution order (recommended)

1. **Skeleton:** T-0137
2. **Content (parallel after T-0137):** T-0138 ∥ T-0139 ∥ T-0140
3. **Runbook (parallel):** T-0142 (may start early)
4. **Validator gate:** T-0141 (after content tasks)
5. **Dev pointer:** T-0143 (after T-0142)

```text
T-0137 → (T-0138 + T-0139 + T-0140) → T-0141
T-0142 → T-0143
```

## Acceptance coverage map

| AC | Tasks | Notes |
|----|-------|-------|
| AC-1 | T-0137, T-0138, T-0139, T-0140 | Split layout + Flow Finance AI content |
| AC-2 | T-0137 | Contributing pointer; no DEV_* in root |
| AC-3 | T-0140 | Related docs + compose commands |
| AC-4 | T-0141 | `--no-template-parity` validator exit 0 |
| AC-5 | T-0142, T-0143 | Runbook hooks + dev shard pointer |
| AC-6 | _(deferred)_ | Vacuous until `template/` tree exists (T1 flip gate) |

## Split decision

- **Why 7 tasks:** Maps architecture execute slices R1–R6; separates content authoring from CI gate and maintenance docs; within SPRINT_MAX_TASKS=12.
- **Why not split S0013a/b:** README content and validator gate share one acceptance contract; splitting would leave CI red between sprints.
- **USER_GUIDE_MODE=1:** No per-story `docs/user-guides/US-0016.md` — architecture defers to root cross-link in T-0140.
