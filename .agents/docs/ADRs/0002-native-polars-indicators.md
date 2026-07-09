# ADR 001: Native Polars Expressions for Technical Indicators

- **Date**: 2026-07-04
- **Status**: Accepted
- **Deciders**: User, Antigravity

## Context

The codebase performs technical analysis indicator calculations (SMA, EMA, RSI, MACD, Bollinger Bands, ATR, ADX, OBV) on historical financial data.
We considered using external rust crates like `ta` or `xta` to calculate these indicators, versus implementing native Polars lazy expressions.

## Decision

We chose to implement these calculations using **Native Polars Expressions** (Option A). We will not use external vector-based indicator crates like `ta` or `xta`.

## Consequences

### Pros
- **Performance**: Calculations run in parallel inside the Polars lazy query engine, avoiding sequential CPU loops and avoiding extraction of columns to raw Rust vectors.
- **Dependency Minimization**: Keeps compile times and binary size low by avoiding external analysis crates.
- **Pipeline Integration**: Aligns cleanly with Polars DataFrame chaining.

### Cons
- **Implementation Complexity**: Standard indicator math (like EMA smoothing) is recursive and requires custom rolling window expressions in Polars DSL.
- **Numerical Edge Cases**: Handling of missing data or weekend gaps must be explicitly managed within Polars queries before calculation.
