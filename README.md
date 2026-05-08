# nexus

The nexus language: spec + translator daemon.

## What's here

- `spec/grammar.md` — the nexus grammar spec
  (text language design).
- `spec/examples/` — illustrative `.nexus` files
  showing the grammar in use.
- `src/` — the daemon implementation. Parses nexus text via
  the Nota grammar, builds
  signal frames, dials
  criome over UDS, serialises replies back to text.

## Architecture

See `ARCHITECTURE.md` for the daemon's role
and code map.

For the project being built: criome's
ARCHITECTURE.md.

## Wire formats

- **Client side** (UDS at `/tmp/nexus.sock`): pure **nexus text**
  in / out. The parser self-delimits on matched parens; replies
  pair to requests by **position** on the connection (FIFO).
- **criome side** (UDS at `/tmp/criome.sock`):
  `signal` rkyv frames
  carrying language IR.

Nexus text is the only non-signal messaging surface in the
sema-ecosystem. Once a request crosses the daemon, it is signal
end-to-end.

## License

[License of Non-Authority](LICENSE.md).
