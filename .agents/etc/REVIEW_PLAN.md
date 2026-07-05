# Line-by-Line Code Review Plan
**Target**: `.agents/etc/line_by_line_review.md`

## Project Context
- **Stack**: Rust (tokio, reqwest, sqlx, polars, plotters, clap, dialoguer, rayon)
- **Size**: ~195KB across 12 source files
- **Output Constraint**: Single output too large for full review. Must split into phases.

## True Internal Module Dependency Graph
```
utils.rs         → (no internal deps)
lib.rs           → module declarations only
export.rs        → (no internal deps)
cache.rs         → (no internal deps; external: chrono, sqlx)
analysis.rs      → (no internal deps; external: chrono, polars)
optimization.rs  → (no internal deps; external: polars)
backtest.rs      → (no internal deps; external: polars, serde)
plot.rs          → use crate::optimization::Portfolio
api/coingecko.rs → use crate::cache::CacheBackend
api/yahoo.rs     → use crate::cache::CacheBackend
ui.rs            → use crate::{api, cache, pipeline}
pipeline.rs      → use crate::{analysis, api, cache, export, plot, backtest}
main.rs          → use cdg::{api, cache, pipeline, ui}
```

**Critical finding**: `analysis.rs`, `backtest.rs`, `optimization.rs`, `export.rs` are functionally pure. They receive dataframes/structs and return results. They do NOT depend on cache or themselves. Only `pipeline.rs` bridges them.

## Phase Execution Order

| Phase | Module(s) | Lines | Internal Deps | Output Estimate |
|-------|-----------|-------|---------------|-----------------|
| 1 | `lib.rs`, `utils.rs` | 47 | None | Small |
| 2 | `export.rs` | 46 | None | Small |
| 3 | `cache.rs` | 171 | None | Small-Medium |
| 4a | `analysis.rs` — parsers + Yahoo + align_datasets | 1-350 | None | Medium | Completed |
| 4b | `analysis.rs` — SMA to Bollinger | 350-481 | None | Medium | Completed |
| 4c | `analysis.rs` — ATR, Stoch, ADX, OBV | 483-681 | None | Medium | Completed |
| 4d | `analysis.rs` — compute_returns + prep_ml | 683-872 | None | Medium | Completed |
| 4e | `analysis.rs` — tests + antipatterns + summary | 875-1161 | None | Medium | Completed |
| 5 | `optimization.rs` | ~442 | None | Medium | Completed |
| 6a | `backtest.rs` — structs + core types | 1-200 | None | Medium | Completed |
| 6b | `backtest.rs` — run_backtest_for_asset | 200-600 | None | Medium | Completed |
| 6c | `backtest.rs` — backtest_portfolio + metrics + JSON | 600-1000 | None | Medium | Completed |
| 6d | `backtest.rs` — helpers + tests | 1000-1411 | None | Medium | Completed |
| 7 | `plot.rs` | ~494 | `crate::optimization` | Medium | Completed |
| 8a | `api/coingecko.rs` | ~356 | `crate::cache` | Medium | Completed |
| 8b | `api/yahoo.rs` | ~200 | `crate::cache` | Small | Completed |
| 9 | `ui.rs` | ~500 | `api`, `cache`, `pipeline` | Medium | Completed |
| 10a | `pipeline.rs` — lines 1-300 | 300 | All | Medium | Completed |
| 10b | `pipeline.rs` — lines 300-500 | 200 | All | Medium | Completed |
| 10c | `pipeline.rs` — lines 500-770 | 270 | All | Medium | Completed |
| 11 | `main.rs` | ~600 | All | Medium | Completed |

**Total**: ~32 subphases across 11 phases. Rating: **E** (TOO BLOATED; target ≤25 subphases)

*Note*: These output estimates assume each phase's review text fits within output limits. The split is precautionary. Phases 6, 10, and 4 are the riskiest for output overflow.

---

## Revised Phase Theta Overview
- **Phase 4 series**: 5 subphases (4a-4e)
- **Phase 6 series**: 4 subphases (6a-6d)
- **Phase 10 series**: 3 subphases (10a-10c)
- **All others**: Single subphase/prompt

## Phase Prompts Remaining

### Phase 1: `lib.rs` + `utils.rs`
> "Perform a line-by-line code review of `C:\Users\lucas\Code\CDG\src\lib.rs` and `C:\Users\lucas\Code\CDG\src\utils.rs`. Document every flaw, dead code, antipattern, ambiguous decision, and simplification opportunity. Cite exact line numbers. Output ONLY a `## Phase 1: lib.rs + utils.rs` section formatted for appending to `.agents/etc/line_by_line_review.md`. Do not edit source files."

### Phase 2: `export.rs`
> "Perform a line-by-line code review of `C:\Users\lucas\Code\CDG\src\export.rs`. Document every flaw, dead code, antipattern, ambiguous decision, and simplification opportunity. Cite exact line numbers. Output ONLY a `## Phase 2: export.rs` section formatted for appending to `.agents/etc/line_by_line_review.md`. Do not edit source files."

### Phase 3: `cache.rs`
> "Perform a line-by-line code review of `C:\Users\lucas\Code\CDG\src\cache.rs`. Document every flaw, dead code, antipattern, ambiguous decision, and simplification opportunity. Cite exact line numbers. Output ONLY a `## Phase 3: cache.rs` section formatted for appending to `.agents/etc/line_by_line_review.md`. Do not edit source files."

### Phase 4: `analysis.rs`
> "Perform a line-by-line code review of `C:\Users\lucas\Code\CDG\src\analysis.rs` (1161 lines). This is a large file containing technical indicators and dataset alignment. Document every flaw, dead code, antipattern, ambiguous decision, and simplification opportunity. Be extremely granular—cite exact line numbers for every issue. Output ONLY a `## Phase 4: analysis.rs` section formatted for appending to `.agents/etc/line_by_line_review.md`. Do not edit source files."

### Phase 5: `optimization.rs`
> "Perform a line-by-line code review of `C:\Users\lucas\Code\CDG\src\optimization.rs` (14624 bytes). Document every flaw, dead code, antipattern, ambiguous decision, and simplification opportunity. Cite exact line numbers. Output ONLY a `## Phase 5: optimization.rs` section formatted for appending to `.agents/etc/line_by_line_review.md`. Do not edit source files."

### Phase 6: `backtest.rs`
> "Perform a line-by-line code review of `C:\Users\lucas\Code\CDG\src\backtest.rs` (49719 bytes, 1411 lines). This is the largest pure-logic module. Document every flaw, dead code, antipattern, ambiguous decision, and simplification opportunity. Be extremely granular—cite exact line numbers for every issue. Output ONLY a `## Phase 6: backtest.rs` section formatted for appending to `.agents/etc/line_by_line_review.md`. Do not edit source files."

### Phase 7: `plot.rs`
> "Perform a line-by-line code review of `C:\Users\lucas\Code\CDG\src\plot.rs` (14260 bytes). This module depends on `crate::optimization::Portfolio`. Document every flaw, dead code, antipattern, ambiguous decision, and simplification opportunity. Cite exact line numbers. Output ONLY a `## Phase 7: plot.rs` section formatted for appending to `.agents/etc/line_by_line_review.md`. Do not edit source files."

### Phase 8: `api/` (`coingecko.rs` + `yahoo.rs`)
> "Perform a line-by-line code review of `C:\Users\lucas\Code\CDG\src\api\coingecko.rs` (12122 bytes) and `C:\Users\lucas\Code\CDG\src\api\yahoo.rs` (5450 bytes). These modules depend on `crate::cache::CacheBackend`. Document every flaw, dead code, antipattern, ambiguous decision, and simplification opportunity. Cite exact line numbers. Output ONLY a `## Phase 8: api/` section formatted for appending to `.agents/etc/line_by_line_review.md`. Do not edit source files."

### Phase 9: `ui.rs`
> "Perform a line-by-line code review of `C:\Users\lucas\Code\CDG\src\ui.rs` (16464 bytes). This module depends on `crate::{api, cache, pipeline}`. Document every flaw, dead code, antipattern, ambiguous decision, and simplification opportunity. Cite exact line numbers. Output ONLY a `## Phase 9: ui.rs` section formatted for appending to `.agents/etc/line_by_line_review.md`. Do not edit source files."

### Phase 10: `pipeline.rs`
> "Perform a line-by-line code review of `C:\Users\lucas\Code\CDG\src\pipeline.rs` (49668 bytes, 770 lines). This is the largest orchestration god-function. Document every flaw, dead code, antipattern, ambiguous decision, and simplification opportunity. Be extremely granular—cite exact line numbers for every issue. Output ONLY a `## Phase 10: pipeline.rs` section formatted for appending to `.agents/etc/line_by_line_review.md`. Do not edit source files."

### Phase 11: `main.rs`
> "Perform a line-by-line code review of `C:\Users\lucas\Code\CDG\src\main.rs` (19384 bytes). This is the CLI entry point. Document every flaw, dead code, antipattern, ambiguous decision, and simplification opportunity. Cite exact line numbers. Output ONLY a `## Phase 11: main.rs` section formatted for appending to `.agents/etc/line_by_line_review.md`. Do not edit source files."

---

## Instructions
Run each prompt sequentially. After each phase, the output appends to `.agents/etc/line_by_line_review.md`. Only `.agents/etc/line_by_line_review.md` is modified across all phases.
