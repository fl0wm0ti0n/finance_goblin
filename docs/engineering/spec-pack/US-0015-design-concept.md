# Design Concept — US-0015

## Summary

US-0015 adds **AI-assisted income/fixed/variable bucket assignment** on the forecast projection path when Firefly categories are missing or ambiguous — extending BUG-0012's DEC-0007 config baseline without overriding mapped rows.

## Goals

- Rule→LLM cascade proposes buckets for ambiguous recurring mirror rows (AC-2 / DEC-0078)
- Config map precedence preserved — AI never overrides DEC-0007 mappings (AC-1)
- Privacy-safe feature extraction under `allow_raw_transactions=false` default (AC-3)
- Monthly API exposes `bucket_sources` + `ai_mapped`; UI badge on Forecast Monthly tab (AC-4/AC-5)
- Audit trail for AI bucket assignments (AC-6)
- Chat tools and US-0013 ML overlay unchanged (AC-7)

## Non-goals

- Firefly write-back or in-app category editing
- RAG/vector merchant index
- `[forecast.merchant_aliases]` TOML (post-MVP)
- Rolling residual aggregate AI split (MVP keeps Variable)
- Seventh chat tool or chat tool registry changes

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0078 | Config→rule→LLM→Variable cascade | Industry privacy-first pattern (R-0074) |
| DEC-0007 | Config map first | BUG-0012 baseline; AC-1 guard |
| DEC-0032 | `allow_raw_transactions=false` | Hashed merchant tokens + magnitude bands (R-0075) |
| US-0008 | Shared `build_provider()` | No forecast-specific provider env |
| Threshold | 0.75 default | Balance German merchant ambiguity vs mis-bucketing risk |

## User experience

Monthly tab retains four Finanzguru-style stat cards. When any bucket total includes AI-assigned recurring mass, an **AI-mapped** badge appears (seasonal callout pattern). Tooltip explains config precedence and privacy-safe inference. Config-only months show no badge.
