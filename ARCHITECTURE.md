# ARCHITECTURE — nexusd

The translator daemon. Speaks nexus text on the client side;
speaks signal rkyv on the criomed side. Holds **no sema state**
— purely a messenger.

```
client (nexus-cli, agents, editors)
   │
   │ client-msg (rkyv around nexus text + control verbs)
   │
   ▼
┌──────────┐
│  nexusd  │   parse text via nota-serde-core (Dialect::Nexus)
│          │   translate AST → signal frames
│          │   relay signal replies back, rendered to text
└────┬─────┘
     │
     │ signal (rkyv envelope around language IR)
     │
     ▼
   criomed
```

## Boundaries

Owns (`[lib]` + `[[bin]]` split):

- **lib half** (`src/client_msg/`): the rkyv envelope between
  *any* client and nexusd. Re-exported so editor LSPs, agent
  harnesses, and nexus-cli all import the same contract types.
- **bin half** (`src/main.rs`): the daemon process — UDS
  listener, parsing, signal connection to criomed, reply
  serialisation.
- The **mechanical translation rule**: every nexus text
  construct has exactly one signal form, and vice versa. Per
  [mentci/reports/070 §7](https://github.com/LiGoldragon/mentci/blob/main/reports/070-nexus-language-and-contract.md).

Does not own:

- The nexus grammar (lives in
  [nexus](https://github.com/LiGoldragon/nexus); parsed by
  [nota-serde-core](https://github.com/LiGoldragon/nota-serde-core)).
- The signal envelope (lives in
  [signal](https://github.com/LiGoldragon/signal)).
- Sema state — that's criomed's exclusive concern.
- The validator pipeline.

## Two messaging surfaces

nexusd is the *only* place where these meet:

| Surface | Direction | Format | Carries |
|---|---|---|---|
| **client-msg** | client ↔ nexusd | rkyv | nexus text payload + control (Heartbeat / Cancel / Resume / fallback file) |
| **signal** | nexusd ↔ criomed | rkyv | language IR (Assert / Mutate / Query / Subscribe / …) |

nexus text is the *only* non-rkyv messaging surface in the
system. It is transient — never persisted, never rendered
outside nexusd.

## Stateless modulo correlations

nexusd holds:

- In-flight `correlation_id` ↔ pending-reply mappings.
- Open subscription streams (one signal frame in, many out).
- A fallback-file dispatch path when a client socket dies
  before its reply lands.

Nothing else. No sema cache, no record knowledge — kind
resolution happens at criomed.

## Code map

```
src/
├── lib.rs                — exposes client_msg as a public module
├── main.rs               — daemon entry, UDS bind, accept loop
└── client_msg/           — the lib-half contract
    ├── mod.rs
    ├── frame.rs          — Frame envelope, RequestId, encode/decode
    ├── request.rs        — Request enum (Send, Heartbeat, Cancel, Resume)
    ├── reply.rs          — Reply enum (Ack, Working, Done, Failed,
    │                        ResumedReply, ResumeNotReady, Cancelled,
    │                        DoneWithFallback, FailedFallback)
    ├── fallback.rs       — FallbackSpec, FallbackFormat
    └── path.rs           — WirePath (raw POSIX bytes)
```

## Invariants

- **Text crosses only at this boundary.** All daemon-to-daemon
  internal traffic is rkyv. No raw nexus text reaches criomed.
- **No state survives a request.** Anything stateful is
  client-state (held by clients via Resume) or criomed-state
  (held in sema). nexusd holds correlation only.
- **rkyv 0.8 portable feature set** for client-msg per
  [mentci/reports/074](https://github.com/LiGoldragon/mentci/blob/main/reports/074-portable-rkyv-discipline.md).

## Status

**Skeleton-as-design.** client-msg types + Frame::encode/decode
shipped; main daemon body lands alongside criomed scaffolding.

## Cross-cutting context

- Three-layer messaging story (client-msg / signal / criome-
  net): [mentci/reports/077](https://github.com/LiGoldragon/mentci/blob/main/reports/077-nexus-and-signal.md)
- Project-wide architecture:
  [criome/ARCHITECTURE.md](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md)
