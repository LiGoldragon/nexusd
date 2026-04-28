# Agent instructions

Repo role: the **nexus text translator daemon** — parses nexus text into [`signal`](https://github.com/LiGoldragon/signal) frames for criome, renders signal replies back to text. Plus the canonical `spec/grammar.md` for the nexus dialect.

Read [ARCHITECTURE.md](ARCHITECTURE.md) for boundaries, code map, and the supervision tree.

Workspace conventions live in [mentci/AGENTS.md](https://github.com/LiGoldragon/mentci/blob/main/AGENTS.md) — beauty, methods on types, full-English naming, `-daemon` binary suffix, S-expression commit messages, jj + always-push.

Ractor patterns (one actor per file, four-piece template, mailbox semantics, supervision) live in [tools-documentation/rust/ractor.md](https://github.com/LiGoldragon/tools-documentation/blob/main/rust/ractor.md).

The post-handshake signal connection ([`CriomeLink`](src/criome_link.rs)) and the text transformers ([`Parser`](src/parser.rs), [`Renderer`](src/renderer.rs)) stay as plain structs — single-owner, no concurrent state. Don't promote them to actors.
