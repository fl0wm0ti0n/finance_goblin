# Technical Specification — US-0016

## Overview

Documentation-only story. Deliverables are markdown surfaces and runbook hooks governed by **DEC-0070** and the existing doc-profile validator. No backend/frontend runtime changes.

## Components

| Component | Path | Role |
|-----------|------|------|
| Root README | `README.md` | User channel: 5 H2s + Contributing + Product status subsection |
| Developer shard | `docs/developer/README.md` | DEV_* H2s; workflow pointer to README maintenance |
| Validator | `scripts/validate_doc_profile.py` | Release gate; `--no-template-parity` while `template/` absent |
| Profile lib | `scripts/doc_profile_lib.py` | H2 budget, optional-mode crosslink checks, template parity |
| Runbook | `docs/engineering/runbook.md` | New § README maintenance (US-0016) |
| CI | `tests/run-tests.sh` or equivalent doc gate | Invoke validator with `--no-template-parity` |

## Interfaces

### Root README structure (`both` / `balanced`)

```text
# Flow Finance AI (or product title)
## Purpose
### Product status          ← DEC-0070: max 8 bullets, reverse-chronological
## Quickstart
## Examples
## Limitations
## Related documentation
## Contributing              ← pointer to docs/developer/README.md only
```

### Product status bullet contract

```text
- US-xxxx — one-line outcome
- BUG-xxxx — one-line outcome
```

Link full history: `docs/product/backlog.md`.

### Validator commands

```bash
# Current (template/ absent)
python scripts/validate_doc_profile.py --repo . --no-template-parity

# After template/ tree ships (same change set as tree)
python scripts/validate_doc_profile.py --repo .
```

### Optional-mode crosslinks (scratchpad)

| Flag | Required mention in root README |
|------|--------------------------------|
| `USER_GUIDE_MODE=1` | `docs/user-guides` |
| `SPEC_PACK_MODE=1` | engineering docs or spec-pack paths |

Non-blocking warnings: `[DOC_OPTIONAL_CROSSLINK_WEAK]` when weak/absent.

### Maintenance hooks (phase boundaries)

| Phase | Trigger | Action |
|-------|---------|--------|
| `/release` | US/BUG → DONE/CLOSED in sprint | Append Product status bullet; trim to 8; run validator |
| `/refresh-context` | Closures since prior refresh | Verify/update Product status; run validator if README touched |

## Non-functional

- **Fail closed:** validator non-zero blocks release readiness
- **No secrets:** compose examples use `.env.example` placeholders only
- **Incremental edits:** avoid full README rewrites each sprint
- **History preservation:** architecture/decisions/runbook append-only patterns unchanged

## Traceability

- Decision: `decisions/DEC-0070.md`
- Architecture: `docs/engineering/architecture.md` § **US-0016**
- Research: R-0066, R-0067
