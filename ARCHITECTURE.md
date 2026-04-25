# ARCHITECTURE — nexus

The nexus language: **spec + translator daemon** in one repo.
This repo holds:

1. [`spec/grammar.md`](spec/grammar.md) — the canonical nexus
   grammar spec.
2. `src/` — the daemon binary `nexus`. Speaks nexus text on the
   client side; speaks signal rkyv on the criome side. Holds
   **no sema state** — purely a messenger.

```
client (nexus-cli, agents, editors)
   │
   │ client-msg (rkyv around nexus text + control verbs)
   │
   ▼
┌──────────┐
│  nexus   │   parse text via nota-serde-core (Dialect::Nexus)
│ (daemon) │   translate AST → signal frames
│          │   relay signal replies back, rendered to text
└────┬─────┘
     │
     │ signal (rkyv envelope around language IR)
     │
     ▼
   criome
```

## Boundaries

Owns (`[lib]` + `[[bin]]` split):

- **The grammar spec** (under [`spec/`](spec/)). Stable;
  changes coordinated with `nota-serde-core`.
- **lib half** (`src/client_msg/`): the rkyv envelope between
  *any* client and the daemon. Re-exported so editor LSPs,
  agent harnesses, and `nexus-cli` all import the same
  contract types via `nexus::client_msg`.
- **bin half** (`src/main.rs`): the daemon process — UDS
  listener, parsing, signal connection to criome, reply
  serialisation.
- The **mechanical translation rule**: every nexus text
  construct has exactly one signal form, and vice versa.

Does not own:

- Lexer/parser kernel (lives in
  [nota-serde-core](https://github.com/LiGoldragon/nota-serde-core)).
- The signal envelope (lives in
  [signal](https://github.com/LiGoldragon/signal)).
- Sema state — that's criome's exclusive concern.
- The validator pipeline.

## Two messaging surfaces

The nexus daemon is the *only* place where these meet:

| Surface | Direction | Format | Carries |
|---|---|---|---|
| **client-msg** | client ↔ nexus | rkyv | nexus text payload + control (Heartbeat / Cancel / Resume / fallback file) |
| **signal** | nexus ↔ criome | rkyv | language IR (Assert / Mutate / Query / Subscribe / …) |

Nexus text is the *only* non-rkyv messaging surface in the
sema-ecosystem. It is transient — never persisted, never
rendered outside this daemon.

## Stateless modulo correlations

The daemon holds:

- In-flight `correlation_id` ↔ pending-reply mappings.
- Open subscription streams (one signal frame in, many out).
- A fallback-file dispatch path when a client socket dies
  before its reply lands.

Nothing else. No sema cache, no record knowledge — kind
resolution happens at criome.

## Code map

```
nexus/
├── spec/
│   ├── grammar.md                — the nexus grammar spec
│   └── example.nexus             — illustrative input
└── src/
    ├── lib.rs                    — exposes client_msg as a public module
    ├── main.rs                   — daemon entry, UDS bind, accept loop
    └── client_msg/               — the lib-half contract
        ├── mod.rs
        ├── frame.rs              — Frame envelope, RequestId, encode/decode
        ├── request.rs            — Request (Send, Heartbeat, Cancel, Resume)
        ├── reply.rs              — Reply (Ack, Working, Done, Failed,
        │                            ResumedReply, ResumeNotReady, Cancelled,
        │                            DoneWithFallback, FailedFallback)
        ├── fallback.rs           — FallbackSpec, FallbackFormat
        └── path.rs               — WirePath (raw POSIX bytes)
```

## Invariants

- **Text crosses only at this boundary.** All daemon-to-daemon
  internal traffic is rkyv. No raw nexus text reaches criome.
- **No state survives a request.** Anything stateful is
  client-state (held by clients via Resume) or criome-state
  (held in sema). The daemon holds correlation only.
- **rkyv 0.8 portable feature set** for client-msg per
  [mentci/reports/074](https://github.com/LiGoldragon/mentci/blob/main/reports/074-portable-rkyv-discipline.md).

## Status

**Skeleton-as-design.** client-msg types + Frame::encode/decode
shipped; main daemon body lands alongside criome scaffolding.

## Cross-cutting context

- Three-layer messaging story (client-msg / signal / criome-
  net): [mentci/reports/077](https://github.com/LiGoldragon/mentci/blob/main/reports/077-nexus-and-signal.md)
- Project-wide architecture:
  [criome/ARCHITECTURE.md](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md)
