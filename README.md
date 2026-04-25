# nexusd

**The nexus messenger daemon.** Translates nexus text ↔ rkyv at
the human boundary. Accepts nexus-syntax messages from clients
(humans, LLMs, `nexus-cli`), parses them into typed rkyv
envelopes, forwards to `criomed`, and serialises replies back to
nexus text.

nexusd does **not** own the database. The sema records database
is owned by `criomed`. Per the sema-ecosystem architecture
(`mentci-next/docs/architecture.md`):

- **nexusd** — messenger: text ↔ rkyv; stateless modulo
  in-flight correlations.
- **criomed** — sema's engine: applies mutations; cascades
  rules; maintains invariants.
- **lojixd** — executor: spawns cargo / nix; manages lojix-store
  blobs.

Wire format (client-facing): nexus syntax, parsed by
[nexus-serde](https://github.com/LiGoldragon/nexus-serde).
Wire format (to criomed): rkyv, over the `signal` contract.

## License

[License of Non-Authority](LICENSE.md).
