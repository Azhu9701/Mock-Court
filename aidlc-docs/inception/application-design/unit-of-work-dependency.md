# Unit of Work Dependencies — 万民幡 Web Application

## Dependency Matrix

| | B1 | B2 | B3 | B4 | B5 | B6 | F1 | F2 | F3 | F4 |
|---|---|---|---|---|---|---|---|---|---|---|
| **B1** | — | | | | | | | | | |
| **B2** | ✓ | — | | | | | | | | |
| **B3** | ✓ | | — | | | | | | | |
| **B4** | ✓ | ✓ | ✓ | — | | | | | | |
| **B5** | ✓ | | | | — | | | | | |
| **B6** | ✓ | ✓ | ✓ | ✓ | ✓ | — | | | | |
| **F1** | | | | | | ✓ | — | | | |
| **F2** | | | | | | ✓ | ✓ | — | | |
| **F3** | | | | ✓ | | ✓ | ✓ | | — | |
| **F4** | | | | | ✓ | ✓ | ✓ | | | — |

## Critical Path

```
B1 ──► B2 ──┐
        B3 ──┼──► B4 ──► B6 ──► F1 ──► F2
        B5 ──┘                        ├──► F3 (also depends on B4 via WS)
                                       └──► F4
```

## Dependency Details

| From | To | Type | Reason |
|------|----|------|--------|
| B2 → B1 | build-time | Storage trait, data models |
| B3 → B1 | build-time | Storage trait, config |
| B4 → B1 | build-time | Data models, config |
| B4 → B2 | build-time | SoulRegistry for soul lookup |
| B4 → B3 | build-time | AI Gateway for soul execution |
| B5 → B1 | build-time | Storage for FS writes + SQLite |
| B6 → B1-B5 | build-time | All services wired into axum |
| F1 → B6 | runtime (REST) | API contract for routing |
| F2 → B6 | runtime (REST) | Soul CRUD endpoints |
| F2 → F1 | build-time | Shared layout components |
| F3 → B6 | runtime (REST + WS) | Possession endpoints + WS stream |
| F3 → F1 | build-time | Shared layout components |
| F4 → B6 | runtime (REST) | Analytics endpoints |
| F4 → F1 | build-time | Shared layout components |

## Parallelization Opportunities

```
Phase 2 (parallel):
  B2 Registry ──┤
  B3 AI Gateway ├──► can run concurrently
  B5 Archive ───┘

Phase 6 (parallel):
  F2 Soul Browser ──┤
  F3 Possession UI ├──► can run concurrently
  F4 Dashboard ─────┘
```

## Communication Contracts

### B6 → Frontend REST API
`/api/souls`, `/api/possess/*`, `/api/archive/*`, `/api/analytics/*`
→ Must be defined before F1-F4 implementation

### B4 → F3 WebSocket
`ws://localhost:{port}/ws/possess/{session_id}`
→ Contract: JSON messages per channel (soul/{name}, synthesis, system)

### Cross-crate APIs (Rust)
Defined by `foundation` traits:
- `Storage` trait → implemented by foundation, consumed by all crates
- `SoulRegistry` trait → defined in registry crate
- `AiProvider` trait → defined in ai-gateway crate
