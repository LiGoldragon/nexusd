# ARCHITECTURE — nexus

The nexus language: **spec + translator daemon** in one repo.

1. [`spec/grammar.md`](spec/grammar.md) — the canonical nexus
   grammar spec.
2. [`spec/examples/`](spec/examples/) — illustrative `.nexus`
   files showing the grammar in use.
3. `src/` — the daemon binary `nexus-daemon` plus the one-shot
   binaries `nexus-parse` and `nexus-render`. The daemon speaks
   **nexus text** on the client side (UDS at
   `/tmp/nexus.sock`) and **signal** (rkyv) on the criome side
   (UDS at `/tmp/criome.sock`); the one-shots are stdin/stdout
   wrappers around the same `Parser` / `Renderer` types for
   test pipelines and agent harnesses. Holds no sema state —
   purely a translator.

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
    ├── lib.rs                    — re-exports + supervision-tree doc
    ├── error.rs                  — typed daemon-process errors (incl. ActorCall, ActorSpawn)
    ├── daemon.rs                 — Daemon actor: root of the supervision tree; spawns Listener
    ├── listener.rs               — Listener actor: UDS accept loop, spawns Connection per accept
    ├── connection.rs             — Connection actor: per-client text shuttle (single-message Run lifecycle)
    ├── criome_link.rs            — CriomeLink struct: post-handshake signal connection (single-owner, not an actor)
    ├── parser.rs                 — Parser struct: text → signal::Request (sigil/delimiter dispatch)
    ├── renderer.rs               — Renderer struct: signal::Reply → text (per-variant dispatch)
    ├── main.rs                   — nexus-daemon entry: env config, Actor::spawn root Daemon, await join handle
    └── bin/
        ├── parse.rs              — nexus-parse: stdin text → length-prefixed Frame on stdout
        └── render.rs             — nexus-render: length-prefixed Frame on stdin → text on stdout
```

The supervision tree:

```text
Daemon (root)
  └── Listener
        ├── Connection × M  (one per accepted UDS client)
        └── ...
```

Per-actor file shape — each actor file exports `<Actor>` (ZST
behaviour marker) + `State` + `Arguments` + `Message`. Parser,
Renderer, and CriomeLink stay plain structs (single-owner,
non-concurrent — they're stateless transformers / one-call-at-a-time
sessions, not components warranting their own mailboxes). The
ractor framework is the project default for components with
state and a message protocol — see [`lore/rust/ractor.md`](https://github.com/LiGoldragon/lore/blob/main/rust/ractor.md).

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

**M0 working.** Daemon is ractor-hosted and verified end-to-end
through `mentci-integration` (text in via `nexus-cli` → signal to
criome → reply rendered to text → delivered to client). The full
demo `(Node "User")` → `(Ok)` and `(| Node @name |)` →
`[(Node "User")]` shuttles correctly through both daemons. M1+
streaming framing and M2+ subscription support land as additive
`Message` variants on the Connection actor.

## What nexus-daemon does — and only that

The nexus daemon does **one thing**: translate between nexus text
and signal. In both directions. Nothing else.

- text in → signal out (parsing)
- signal in → text out (rendering)

It does not hold sema state. It does not validate beyond
syntactic well-formedness. It does not remember anything across
requests except the per-connection handshake/subscription state
needed to keep ordered replies flowing. It does not read records,
write records, or know what a record means. Translation is the
whole job.

This bright-line scope makes the daemon usable in two distinct
roles, both reducing to the same translation primitive:

- **Text-client gateway.** Humans, agents, shell scripts, and
  LLM tools send nexus text to the daemon's UDS; the daemon
  parses to signal, forwards to criome, renders the reply back
  to text.
- **Rendering service.** Other signal-speaking clients (mentci-*
  GUIs being the first) speak signal directly to criome but
  also hold a connection to the nexus daemon and use it to
  render typed signal payloads as nexus text for human display
  (inspector views, wire-pane frame display, hover labels). The
  same daemon, the same primitive, used as a service rather
  than a gateway.

In both roles the daemon is a translator. New clients do not
extend nexus's scope — they consume the translation primitive
from whichever side they need.

## Clients connecting directly to criome

Anything that wants to *talk to* criome speaks signal. The nexus
daemon is one such speaker (forwarding the parsed signal from
its text clients). Others connect directly:

- **mentci-* GUIs** — speak signal to criome for editing and
  subscriptions; *additionally* speak signal to nexus-daemon for
  rendering as described above.
- **direct signal speakers** — agents written in Rust, CI
  harnesses, integration tests, any binary linking signal.

Nexus is not a required intermediary for criome access; it is
the text translation primitive that humans and rendering clients
both consume.

## Parser + renderer wire-in for new verbs

Adding a new signal verb (the planned `Compile` / `BuildRequest`
post-MVP, plus any future verb) lands in three places:

1. The verb's typed payload + closed-enum variant in
   [signal](https://github.com/LiGoldragon/signal).
2. A new arm in [`Parser`](src/parser.rs) — sigil/delimiter
   dispatch from the surface text construct to the typed
   payload (Pascal-named record verb head; pattern-matched
   payload).
3. A new arm in [`Renderer`](src/renderer.rs) — typed reply →
   nexus text, one canonical rendering per typed shape.

The mechanical-translation rule (one text construct, one typed
value) extends to every verb the daemon translates; new verbs
slot into the existing parser/renderer per-variant dispatch
without adding new sigils or grammar slots.

## Cross-cutting context

- Project-wide architecture:
  [criome/ARCHITECTURE.md](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md)
- Signal (the rkyv form on the criome leg):
  [signal/ARCHITECTURE.md](https://github.com/LiGoldragon/signal/blob/main/ARCHITECTURE.md)
- nota-codec (text codec used both for parsing client requests
  and rendering replies):
  [nota-codec/ARCHITECTURE.md](https://github.com/LiGoldragon/nota-codec/blob/main/ARCHITECTURE.md)
