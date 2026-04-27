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
│  nexus   │   parse text via nota-codec (Decoder::nexus)
│ (daemon) │   build signal frames, send to criome
│          │   receive signal replies, render to text via nota-codec (Encoder::nexus)
└────┬─────┘
     │
     │ signal (rkyv envelope around per-verb typed payloads)
     │
     ▼
   criome
```

## Boundaries

Owns (`[lib]` + `[[bin]]` split):

- **The grammar spec** (under [`spec/`](spec/)). Stable;
  changes coordinated with
  [nota-codec](https://github.com/LiGoldragon/nota-codec).
- **bin half** (`src/main.rs`): the daemon process — UDS
  listener at `/tmp/nexus.sock`, parsing, signal connection
  to criome, reply rendering.
- **lib half** (`src/lib.rs` + `src/error.rs`): daemon-specific
  helpers — typed errors and (soon) connection-state types
  + a request-routing actor.
- The **mechanical translation rule**: every nexus text
  construct has exactly one signal form, and vice versa.

Does not own:

- Lexer / Decoder / Encoder kernel — lives in
  [nota-codec](https://github.com/LiGoldragon/nota-codec).
  Per-kind parsing of records, pattern records, verbs, and
  primitives is performed by the derives in
  [nota-derive](https://github.com/LiGoldragon/nota-derive)
  (`NotaRecord`, `NotaEnum`, `NotaTransparent`,
  `NotaTryTransparent`, `NexusPattern`, `NexusVerb`) which
  signal types apply.
- The signal envelope and per-verb typed IR — lives in
  [signal](https://github.com/LiGoldragon/signal).
- Sema state — that's criome's exclusive concern.
- The validator pipeline.

## Two messaging surfaces

The nexus daemon is the *only* place where these meet:

| Surface | Direction | Format | Contents |
|---|---|---|---|
| **client-facing** | client ↔ nexus | pure nexus text | the user's nexus expressions in / replies out |
| **signal** | nexus ↔ criome | rkyv | language IR (`AssertOperation` / `MutateOperation` / `QueryOperation` / `Subscribe` / …) |

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
    ├── lib.rs                    — daemon library half + re-exports
    ├── error.rs                  — typed daemon-process errors
    │                              (Io / Codec [from nota_codec::Error] /
    │                               Frame [from signal::FrameDecodeError])
    └── main.rs                   — daemon entry: UDS bind on
                                    /tmp/nexus.sock, accept loop,
                                    per-connection signal client to criome
```

The previous `src/parse.rs` (the hand-written `QueryParser`)
was deleted when nota-codec's `NexusPattern` derive landed.
The same dispatch happens automatically per `*Query` type.

## Invariants

- **Text crosses only at this boundary.** All daemon-to-daemon
  internal traffic is signal (rkyv). No raw nexus text reaches
  criome.
- **No state survives a request.** Per-connection state dies
  with the connection; durable state lives in criome's sema.
- **No correlation IDs.** Position pairs replies to requests.
- **One text construct, one typed value.** The mechanical
  translation rule is the [perfect-specificity
  invariant](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md#invariant-d)
  seen at the text↔signal boundary. Every nexus text construct
  names exactly one typed shape; every typed shape has exactly
  one canonical text rendering. The daemon never instantiates
  a generic record and figures out its kind later — it parses
  text directly into the precise typed payload of the verb
  the text expresses (`AssertOperation::Node(node)`,
  `MutateOperation::Edge { slot, new, expected_rev }`,
  `QueryOperation::Graph(GraphQuery{…})`). Failure to parse
  into a known kind is a parse-time error, not a downstream
  validation miss.

## Status

**Skeleton.** Grammar spec is locked; example .nexus files
exist; daemon body (UDS bind + per-connection text shuttle +
paired criome connection + reply rendering) lands alongside
the criome body — see
[mentci/reports/089 step 5](https://github.com/LiGoldragon/mentci/blob/main/reports/089-m0-implementation-plan-step-3-onwards.md).
The codec primitives and derives are ready; the daemon body
just wires them up.

## Cross-cutting context

- Project-wide architecture:
  [criome/ARCHITECTURE.md](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md)
- Signal (the rkyv form on the criome leg):
  [signal/ARCHITECTURE.md](https://github.com/LiGoldragon/signal/blob/main/ARCHITECTURE.md)
- nota-codec (text codec used both for parsing client requests
  and rendering replies):
  [nota-codec/ARCHITECTURE.md](https://github.com/LiGoldragon/nota-codec/blob/main/ARCHITECTURE.md)
