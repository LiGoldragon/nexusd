# ARCHITECTURE — nexus

The nexus language: **spec + translator daemon** in one repo.

1. [`spec/grammar.md`](spec/grammar.md) — the canonical nexus
   grammar spec.
2. [`spec/examples/`](spec/examples/) — illustrative `.nexus`
   files showing the grammar in use.
3. `src/` — the daemon binary `nexus`. Speaks **nexus text** on
   the client side (UDS at `/tmp/nexus.sock`); speaks **signal**
   (rkyv) on the criome side (UDS at `/tmp/criome.sock`). Holds
   no sema state — purely a translator.

```
client (nexus-cli, agents, editors, shell scripts)
   │
   │ pure nexus text in / out
   │
   ▼
┌──────────┐
│  nexus   │   parse text via nota-serde-core (Dialect::Nexus)
│ (daemon) │   build signal frames, send to criome
│          │   receive signal replies, render to text
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
- **bin half** (`src/main.rs`): the daemon process — UDS
  listener at `/tmp/nexus.sock`, parsing, signal connection
  to criome, reply rendering.
- **lib half** (`src/lib.rs` + `src/error.rs`): daemon-
  specific helpers (errors, daemon-state types). The wire
  protocol on both sides lives elsewhere — nexus text is
  defined by the grammar spec; signal frames are defined in
  the [signal](https://github.com/LiGoldragon/signal) crate.
- The **mechanical translation rule**: every nexus text
  construct has exactly one signal form, and vice versa.

Does not own:

- Lexer/parser kernel (lives in
  [nota-serde-core](https://github.com/LiGoldragon/nota-serde-core)).
- The signal envelope and IR (lives in
  [signal](https://github.com/LiGoldragon/signal)).
- Sema state — that's criome's exclusive concern.
- The validator pipeline.

## Two messaging surfaces

The nexus daemon is the *only* place where these meet:

| Surface | Direction | Format | Contents |
|---|---|---|---|
| **client-facing** | client ↔ nexus | pure nexus text | the user's nexus expressions in / replies out |
| **signal** | nexus ↔ criome | rkyv | language IR (Assert / Mutate / Query / Subscribe / …) |

Nexus text is the only non-signal messaging surface in the
sema-ecosystem. It is transient — never persisted, never
rendered outside this daemon.

## Per-connection state

The daemon holds, per open connection:

- The negotiated protocol version (from the handshake).
- Open subscription registration (one subscription per
  connection; events stream until close).

Nothing else. No correlation-id mappings (replies pair to
requests by **position** on the connection — FIFO). No
fallback-file dispatch. No resume after disconnect (durable
work is criome-state, fetched via Query). No sema cache.

## Code map

```
nexus/
├── spec/
│   ├── grammar.md                — the canonical nexus grammar
│   └── examples/                 — illustrative .nexus files
└── src/
    ├── lib.rs                    — daemon library half
    ├── error.rs                  — daemon error types
    └── main.rs                   — daemon entry, UDS bind, accept loop
```

## Invariants

- **Text crosses only at this boundary.** All daemon-to-daemon
  internal traffic is signal (rkyv). No raw nexus text reaches
  criome.
- **No state survives a request.** Per-connection state dies
  with the connection; durable state lives in criome's sema.
- **No correlation IDs.** Position pairs replies to requests.

## Status

**Skeleton-as-design.** Grammar spec is locked; example .nexus
files exist; daemon body lands alongside criome scaffolding.

## Cross-cutting context

- Reply protocol: [mentci/reports/083](https://github.com/LiGoldragon/mentci/blob/main/reports/083-the-return-protocol.md)
- Three-layer messaging story:
  [mentci/reports/077](https://github.com/LiGoldragon/mentci/blob/main/reports/077-nexus-and-signal.md)
- Project-wide architecture:
  [criome/ARCHITECTURE.md](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md)
