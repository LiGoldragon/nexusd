# Agent instructions — nexus

You **MUST** read AGENTS.md at `github:ligoldragon/lore` — the workspace contract.

## Repo role

The **Nexus translator daemon** — parses NOTA text containing Nexus records into Signal frames for criome, then renders Signal replies back to NOTA text. `spec/grammar.md` is the canonical Nexus vocabulary/spec over NOTA syntax, not a separate text syntax.

Ractor patterns (one actor per file, four-piece template, mailbox semantics, supervision) live in lore/rust/ractor.md.

---

## Carve-outs worth knowing

- The post-handshake signal connection ([`CriomeLink`](src/criome_link.rs)) and the text transformers ([`Parser`](src/parser.rs), [`Renderer`](src/renderer.rs)) stay as plain structs — single-owner, no concurrent state. Keep them outside the actor system.
