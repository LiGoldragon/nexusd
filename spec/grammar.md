# nexus

A messaging protocol built on [nota](https://github.com/LiGoldragon/nota).
Adds sigils, delimiter pairs, and the `=` token for **pattern,
mutate, bind, negate, shape, and alias** actions against a sema
world-store. The Rust implementation is
[nexus-serde](https://github.com/LiGoldragon/nexus-serde).

Every valid `.nota` text is also valid `.nexus`; the reverse is
not true — nexus adds constructs that `.nota` parsers reject.

This repo is spec-only.

---

## Inherited from nota

See [nota's spec](https://github.com/LiGoldragon/nota) for the full
grammar. nexus inherits:

- 4 delimiter pairs: `( )` records (positional), `[ ]` / `[| |]`
  strings, `< >` sequences
- 2 sigils: `;;` line comments, `#` byte-literal prefix
- 3 identifier classes: PascalCase (types/variants), camelCase
  (fields in schema / instance names), kebab-case (titles / tags)
- Integer / float / bool / bytes / string literals, `:` path syntax
- Bare-identifier string form: when a schema expects a String,
  an ident-class token (non-reserved) may be written in place
  of `[identifier]` — e.g. `@horizontal` is a bind, but the
  string value of a String-typed field `name` can be written
  bare as `nota-serde` instead of `[nota-serde]`

Records are positional: `(TypeName v1 v2 …)`. Field names live
in the schema, not the text.

---

## Added by nexus

**3 additional delimiter pairs:**

| Pair | Role | Example |
|---|---|---|
| `(\| \|)` | Pattern — matches a record by shape | `(\| Point @horizontal @vertical \|)` |
| `{\| \|}` | Constrain — conjunction of patterns | `{\| (\| Point @h @v \|) (\| Positive @h \|) \|}` |
| `{ }` | Shape — projection / field selection | `{ horizontal vertical }` |

**3 additional sigils:**

| Sigil | Role | Position |
|---|---|---|
| `~` | Mutate marker — "replace / overwrite this" | prefix on a title, record, or pattern |
| `@` | Bind marker — names a hole the matcher fills | prefix on an identifier in a pattern |
| `!` | Negate — invert the match or fact | prefix on a value or pattern |

**One extra token:**

| Token | Role |
|---|---|
| `=` | Bind-alias separator. Valid only between two bind names (`@a=@b`). Not valid as a field-value separator. |

Total: 7 delimiter pairs, 4 sigils, `=` in its narrow role.
First-token-decidable at every choice point; no interior
scanning.

---

## Binds — auto-named from schema

A bind is a named hole in a pattern. The matcher walks records,
finds shape-matching ones, and for each match records the actual
values at bound positions.

The bind's name comes from the schema field at that position:

```nexus
(| Point @horizontal @vertical |)
```

For `struct Point { horizontal: f64, vertical: f64 }`, this
matches any Point and binds `@horizontal` and `@vertical` to the
actual values. No field names appear in the text other than
through the `@`-sigil form.

### `@data` for newtype inner

Newtype structs (`struct Id(u32)`) have an unnamed inner value.
The reserved bind name `@data` names it:

```nexus
(| Id @data |)
```

Matches any `Id` record, binds its inner `u32` to `@data`.

### `_` wildcard

Bare `_` matches any value at that position without binding:

```nexus
(| Point 3.0 _ |)
```

Matches Points whose horizontal is exactly `3.0` and whose
vertical is anything (not bound).

### Bind aliasing — `@a=@b`

When you need two positions to unify (classic non-linear
pattern), alias one bind name to a second:

```nexus
(| Pair @left=@right |)
```

Pos 1 (schema name `left`) is bound as both `@left` and `@right`.
If `@right` is referenced elsewhere in the query (same pattern or
another pattern in a conjunction), all occurrences must unify —
forcing equality.

Aliasing is the only place `=` appears in nexus. Anywhere else
it's a syntax error.

### Concrete values, binds, and wildcards mix freely

```nexus
(| Shape (Circle @radius) |)     ;; match Circle, bind radius
(| Shape (Square @data) |)       ;; match Square newtype, bind inner
(| Shape Active |)               ;; match the unit variant
(| Config 0 _ @strict |)         ;; pos 1 = 0; pos 2 any; bind pos 3
```

---

## Messaging shapes

Five first-class shapes, composed from the inherited + added
constructs.

### Assert — a plain record

```nexus
(Point 3.0 4.0)
```

Identical to nota. An assertion that the record exists / is true.

### Observe — a pattern

```nexus
(| Point @horizontal @vertical |)
```

Matches records of type `Point`; returns all binding tuples.

### Shape — a projection

```nexus
(| Point @horizontal @vertical |) { horizontal }
```

Match, then project — "give me only `horizontal` of each
matching record." Shapes can be applied to any pattern.

### Mutate — `~` on a record or pattern

```nexus
~(Point 0.0 0.0)
```

Asserts the record, replacing any prior record with the same
identity.

```nexus
~(| Point @horizontal @vertical |) (0.0 @vertical)
```

For each match, overwrite with a new record: horizontal = 0.0,
vertical unchanged from the bound value. The replacement record
is positional, just like any other record.

### Negate — `!` on a value, record, or pattern

```nexus
!(Active)
```

Retract the fact — `Active` is no longer asserted.

```nexus
(| Point !0.0 @vertical |)
```

Matches `Point` records whose horizontal is *not* zero; binds
vertical.

---

## Constraining multiple patterns

```nexus
{|
  (| Point @horizontal @vertical |)
  (| Positive @horizontal |)
  (| Positive @vertical |)
|}
```

All three patterns must match simultaneously — the constrain
delimiter is logical conjunction over its contents. Shared bind
names (`@horizontal` here) unify across patterns.

---

## Canonical form

Inherited from nota:

- Source-declaration field order
- Sorted map keys
- Shortest-roundtrip numbers with mandatory `.` in floats
- Single-space expression separators
- `#`-prefixed lowercase hex bytes
- Records positional, no field-name text

nexus-specific:

- `~`, `@`, `!` sit immediately before their target with no
  intervening whitespace (`~(Point …)`, `@h`, `!0.0`).
- `=` in `@a=@b` has no surrounding whitespace.
- `_` is a bare token in pattern position; not valid as a value.

---

## Implementation

[nexus-serde](https://github.com/LiGoldragon/nexus-serde) implements
`serde::Serializer` + `serde::Deserializer` for the full grammar.
Consumer code (e.g. [nexusd](https://github.com/LiGoldragon/nexusd),
[nexus-cli](https://github.com/LiGoldragon/nexus-cli)) derives
serde on message types and round-trips them through nexus text.

For pure-data configs that don't need the query layer,
[nota-serde](https://github.com/LiGoldragon/nota-serde) is the
leaner choice. A nota document round-trips through nexus-serde
too, since the grammar is a strict superset.

---

## Repo layout

```
nexus/
  README.md            ;; this file — grammar spec
  flake.nix            ;; dev-shell
  LICENSE.md
```
