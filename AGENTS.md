# Agent instructions — nexus

You **MUST** read AGENTS.md at `github:ligoldragon/lore` — the workspace contract.

## Repo role

The **Nexus translator daemon** — parses NOTA text containing Nexus records into Signal frames for criome, then renders Signal replies back to NOTA text. `spec/grammar.md` is the canonical Nexus vocabulary/spec over NOTA syntax, not a separate text syntax.

Actor patterns follow the workspace's Kameo discipline: data-bearing actor
types, typed messages, mailbox semantics, and supervision live in
`~/primary/skills/actor-systems.md` and `~/primary/skills/kameo.md`.

---

## Carve-outs worth knowing

- The post-handshake signal connection ([`CriomeLink`](src/criome_link.rs)) and the text transformers ([`Parser`](src/parser.rs), [`Renderer`](src/renderer.rs)) stay as plain structs — single-owner, no concurrent state. Keep them outside the actor system.
