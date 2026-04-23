# nexusd

The sema database daemon. Receives nexus messages, applies edits to
the database, serves queries.

Ractor-hosted services own the daemon's concurrent state.

Wire format: nexus syntax, deserialized via
[nexus-serde](https://github.com/LiGoldragon/nexus-serde) to
structured operations.

## License

[License of Non-Authority](LICENSE.md).
