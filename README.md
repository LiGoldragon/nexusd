# nexus

The nexus language: spec + translator daemon.

## What's here

- [`spec/grammar.md`](spec/grammar.md) — the nexus grammar spec
  (text language design).
- [`spec/example.nexus`](spec/example.nexus) — illustrative
  nexus text.
- `src/` — the daemon implementation. Parses nexus text via
  `nota-serde-core` at `Dialect::Nexus`, builds [signal](https://github.com/LiGoldragon/signal)
  frames, dials criome over UDS, serialises replies back to
  text.
- `src/client_msg/` — the rkyv envelope between *any* client
  and the daemon (re-exported as `nexus::client_msg`).

## Architecture

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the daemon's role
and code map.

For the project being built: [criome's
ARCHITECTURE.md](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md).

## Wire formats

- **Client side** (UDS): `client-msg` rkyv envelope around
  nexus text + control verbs (Heartbeat / Cancel / Resume).
  Lib half (`nexus::client_msg`) exposed for clients.
- **criome side** (UDS): [`signal`](https://github.com/LiGoldragon/signal)
  rkyv frames carrying language IR.

Nexus text is the *only* non-rkyv messaging surface in the
sema-ecosystem.

## License

[License of Non-Authority](LICENSE.md).
