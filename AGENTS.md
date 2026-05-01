# Agent instructions — nexus

You **MUST** read AGENTS.md at `github:ligoldragon/lore` — the workspace contract.

## Repo role

The **nexus text translator daemon** — parses nexus text into signal frames for criome, renders signal replies back to text. Plus the canonical `spec/grammar.md` for the nexus dialect.

Ractor patterns (one actor per file, four-piece template, mailbox semantics, supervision) live in lore/rust/ractor.md.

---

## Carve-outs worth knowing

- The post-handshake signal connection ([`CriomeLink`](src/criome_link.rs)) and the text transformers ([`Parser`](src/parser.rs), [`Renderer`](src/renderer.rs)) stay as plain structs — single-owner, no concurrent state. Keep them outside the actor system.
