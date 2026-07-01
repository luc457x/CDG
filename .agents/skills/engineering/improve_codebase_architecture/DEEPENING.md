# Deepening

How to deepen a cluster of shallow modules safely, given its dependencies. Assumes vocabulary in [LANGUAGE.md](./LANGUAGE.md) — **module**, **interface**, **seam**, **adapter**.

## Dependency Categories

When assessing candidate for deepening, classify its dependencies. Category determines how deepened module is tested across seam.

### 1. In-Process

Pure computation, in-memory state, no I/O. Always deepenable — merge modules and test through new interface directly. No adapter needed.

### 2. Local-Substitutable

Dependencies with local test stand-ins (PGLite for Postgres, in-memory filesystem). Deepenable if stand-in exists. Deepened module tested with stand-in running in test suite. Seam internal; no port at module's external interface.

### 3. Remote but Owned (Ports & Adapters)

Own services across network boundary (microservices, internal APIs). Define **port** (interface) at seam. Deep module owns logic; transport injected as **adapter**. Tests use in-memory adapter. Production uses HTTP/gRPC/queue adapter.

Recommendation shape: _"Define port at seam, implement HTTP adapter for production and in-memory adapter for testing, so logic sits in one deep module even though deployed across network."_

### 4. True External (Mock)

Third-party services (Stripe, Twilio, etc.) you don't control. Deepened module takes external dependency as injected port; tests provide mock adapter.

## Seam Discipline

- **One adapter = hypothetical seam. Two adapters = real one.** Don't introduce port unless at least two adapters justified (typically production + test). Single-adapter seam = just indirection.
- **Internal seams vs external seams.** Deep module can have internal seams (private to implementation, used by own tests) as well as external seam at interface. Don't expose internal seams through interface just because tests use them.

## Testing Strategy: Replace, Don't Layer

- Old unit tests on shallow modules become waste once tests at deepened module's interface exist — delete them.
- Write new tests at deepened module's interface. **Interface is test surface.**
- Tests assert on observable outcomes through interface, not internal state.
- Tests survive internal refactors — describe behaviour, not implementation. If test must change when implementation changes, it's testing past the interface.
