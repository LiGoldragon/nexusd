# nexus

A messaging protocol built on [nota](https://github.com/LiGoldragon/nota).
Adds sigils and delimiter pairs for **assert / mutate / retract /
validate / query / subscribe / pattern / shape / constrain /
atomic-batch** actions against a record graph.

Every valid `.nota` text is also valid `.nexus`; the reverse is
not true — nexus adds constructs that `.nota` parsers reject.

This repo is spec-only.

---

## Inherited from nota

See [nota's spec](https://github.com/LiGoldragon/nota) for the full
grammar. nexus inherits:

- **2 delimiter pairs**: `( )` records, `[ ]` sequences
- **2 string forms**: `" "` inline, `""" """` multiline
- **2 sigils**: `;;` line comments, `#` byte-literal prefix.
  *Comments carry no load-bearing data — the parser discards
  them. Information that must be communicated has a typed home
  in the schema.*
- **3 identifier classes**: PascalCase (types/variants),
  camelCase (fields in schema / instance names), kebab-case
  (titles / tags)
- Integer / float / bool / bytes literals, `:` path syntax
- Bare-identifier strings (when the schema expects a String,
  an ident-class token may be written bare)

Records are positional: `(TypeName v1 v2 …)`. Field names live
in the schema, not the text.

---

## Added by nexus

### 4 additional delimiter pairs

The three families (round, square, curly) each have a piped
form that means *combine-as-unit*:

| Pair | Role | Example |
|---|---|---|
| `(\| \|)` | **Pattern** — match a record by shape | `(\| Point @h @v \|)` |
| `[\| \|]` | **Atomic batch** — AND of edits (all-or-nothing) | `[\| (Node a "A") ~(Node b "B") \|]` |
| `{ }` | **Shape** — projection / field selection | `{ horizontal vertical }` |
| `{\| \|}` | **Constrain** — conjunction of patterns | `{\| (\| Point @h @v \|) (\| Positive @h \|) \|}` |

Reads as: *plain delimiter = passive data; piped delimiter =
combine the contained items as a single logical unit*.

### 5 additional sigils

| Sigil | Role | Position |
|---|---|---|
| `~` | Mutate / replace | prefix on a record or pattern |
| `!` | Retract / negate | prefix on a record, pattern, or value-in-pattern |
| `?` | Validate (dry-run) | prefix on any verb |
| `*` | Subscribe (continuous query) | prefix on a pattern |
| `@` | Bind hole | prefix on an identifier in a pattern |

### 1 extra token (narrow use)

| Token | Role |
|---|---|
| `=` | Bind-alias separator. Valid only between two bind names (`@a=@b`). Not valid as a field-value separator. |

### Reserved tokens (deferred design)

These tokens are reserved for future use and are syntax errors
today:

- `<` `>` `<=` `>=` `!=` — comparison operators (intended for
  pattern positions: `(\| Person @age (@age < 21) \|)` or similar
  form, design pending).
- `=` between non-bind tokens — equality comparison (the
  bind-alias use is the *only* `=` use today).

### Totals

- **6 delimiter pairs** (2 from nota + 4 from nexus) + **2 string forms**
- **7 sigils** (2 from nota + 5 from nexus)
- **1 narrow-use token** (`=` for bind aliasing)
- First-token-decidable at every choice point; no interior
  scanning.

---

## Verbs

A nexus expression at the top level is *one verb*. The verb is
determined by the leading sigil + delimiter:

| Form | Verb | What it does |
|---|---|---|
| `(R …)` | Assert | State a fact: this record exists |
| `~(R …)` | Mutate | Replace the record at this identity |
| `!(R …)` | Retract | Remove the record |
| `?(R …)` | Validate | Dry-run: would this assert succeed? |
| `~(\| pat \|) (R …)` | Mutate-with-pattern | For each match, overwrite |
| `!(\| pat \|)` | Retract-matching | Remove matching records |
| `?(\| pat \|)` | Validate query | Would the query return anything? |
| `(\| pat \|)` | Query | Find matching records (one-shot) |
| `*(\| pat \|)` | Subscribe | Match continuously; stream events |
| `[\| op1 op2 … \|]` | Atomic batch | Run ops atomically (each carries its own sigil) |

Patch is expressed as Mutate-with-pattern that preserves the
unchanged fields:

```nexus
~(| Node @id _ |) (Node @id "new label")   ;; keep id, replace label
```

---

## Binds — names come from the schema, not the author

A bind is a named hole in a pattern. The matcher walks records,
finds shape-matching ones, and for each match records the actual
values at bound positions.

### The strict rule

**The bind name at any position MUST equal the schema field name
at that position.** The author cannot pick a different name. The
parser rejects any other name with a clear error.

```nexus
;; struct Point { horizontal: f64, vertical: f64 }
(| Point @horizontal @vertical |)   ;; ✓ both names match the schema
(| Point @h @v |)                   ;; ✗ rejected: position 1 expects @horizontal
(| Point @x @y |)                   ;; ✗ rejected: position 1 expects @horizontal
```

This is what makes the IR — `PatternField::Bind` — payload-free.
The bind's "name" lives in the *Query record's field position;
the text just confirms it. Aliasing (below) is the only way to
introduce additional names, and even then the *first* name must
match the schema.

### Why this rule exists

The auto-name rule is a manifestation of the project-wide
[perfect-specificity invariant](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md#invariant-d):
the IR carries no redundant data; field-position carries the
binding identity; the text is a literal reading of that identity.
Allowing `@h` for `@horizontal` would fork "what the parser sees"
from "what the IR records," requiring a mapping table — which is
exactly the kind of indirection the invariant rules out.

### Binds **must** carry the `@` sigil

Inside a pattern, every bind is `@<schema-field-name>`. A bare
lowercase identifier in a pattern position is *not* an implicit
bind — it is a bare-string literal matched by value equality
(consistent with bare-identifier strings elsewhere in nota). The
`@` sigil exists exactly to disambiguate bind from bare-string in
pattern position.

```nexus
(| Tag @name |)        ;; ✓ bind on the `name` field; captures the string
(| Tag name |)         ;; ✓ literal: match Tags whose `name` field equals "name"
(| Tag wrongname |)    ;; ✓ literal: match Tags whose `name` field equals "wrongname"
(| Tag @wrongname |)   ;; ✗ rejected — schema field is `name`, not `wrongname`
```

### Bind name lexical class

Bind names — being schema field names — are camelCase or
kebab-case (lowercase or `_` lead). PascalCase is reserved for
type/variant names; `@Foo` is a parse error.

```nexus
(| Edge @from @to @kind |)   ;; ✓ all three field names are lowercase
(| Edge @From @to @kind |)   ;; ✗ @From — uppercase leader is illegal
```

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

## Atomic batches

Multiple edits processed all-or-nothing:

```nexus
[|
  (Node a "Apple")
  ~(Node b "Banana")
  !(Node c "Cherry")
|]
```

Each item carries its own verb sigil. The batch succeeds only if
every item succeeds; on any failure, no item is applied.

---

## Reply semantics

Replies are messages — typed records — paired to requests by
**position**. The N-th reply on a connection corresponds to the
N-th request. There are no correlation IDs.

Two message kinds carry every reply:

- **`(Ok)`** — success acknowledgement. Empty record, no fields.
- **`(Diagnostic …)`** — failure with reasons (level, code, site,
  suggestions).

### Per-verb shapes

| Request | Reply at the same position |
|---|---|
| `(R …)` Assert | `(Ok)` on success, `(Diagnostic …)` on failure |
| `~(R …)` Mutate | `(Ok)` or `(Diagnostic …)` |
| `!(R …)` Retract | `(Ok)` or `(Diagnostic …)` |
| `?(R …)` Validate | `(Ok)` if the operation would succeed; `(Diagnostic …)` if it would fail |
| `~(\| pat \|) (R …)` Mutate-with-pattern | `[(Ok) (Ok) (Diagnostic …) (Ok) …]` — one outcome per matched record |
| `!(\| pat \|)` Retract-matching | `[(Ok) (Ok) (Diagnostic …) …]` — one outcome per matched record |
| `(\| pat \|)` Query | `[<r1> <r2> …]` — sequence of matching records (empty `[]` for zero matches) |
| `[\| op1 op2 … \|]` Atomic batch | `[(Ok) (Ok) (Diagnostic …) (Ok)]` — one outcome per item in the batch; if any element is a `Diagnostic`, the whole batch rolled back atomically |

The reply distinguishes by content: a sequence of `(Ok)` /
`(Diagnostic)` is an edit-outcome reply; a sequence of records is
a query reply.

### Subscriptions

`*(\| pat \|)` opens a subscription. **One subscription per
connection.** The connection enters streaming mode; events arrive
as they happen, each reusing the request-side sigil discipline:

```
(Node u "User")           ← a new record matched
~(Node u "User updated")  ← a matching record was mutated
!(Node u "User updated")  ← a matching record was retracted
```

There is **no initial snapshot** in the subscribe reply — issue a
separate `(\| pat \|)` Query first if the client wants current
state. End of subscription = client closes the socket; daemon
reaps the subscription. No explicit Unsubscribe message.

---

## Connection semantics

- Client and daemon exchange nexus expressions over a stream
  socket; the parser self-delimits on matched parens.
- Requests and replies are strictly FIFO ordered on a single
  connection. No correlation IDs.
- One subscription per connection. For multiple subscriptions,
  open multiple connections.
- Close the socket to end. No graceful-goodbye message.

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

- `~`, `!`, `?`, `*`, `@` sit immediately before their target
  with no intervening whitespace (`~(Point …)`, `@h`, `!0.0`,
  `?(Node …)`, `*(\| Node @id \|)`).
- `=` in `@a=@b` has no surrounding whitespace.
- `_` is a bare token in pattern position; not valid as a value.

---

## Implementation

[nota-codec](https://github.com/LiGoldragon/nota-codec) +
[nota-derive](https://github.com/LiGoldragon/nota-derive)
provide the typed Decoder + Encoder + six derive macros that
map any record kind to its wire form. Consumer code derives
the appropriate Nota / Nexus derive on message types and
round-trips them through nexus text.

The dialect knob (`Decoder::nexus(text)` vs
`Decoder::nota(text)`) selects the grammar. nota-only types
that derive `NotaRecord` / `NotaEnum` / `NotaTransparent`
round-trip in either dialect; types deriving `NexusPattern`
or `NexusVerb` are nexus-only.

---

## Repo layout

```
nexus/
  spec/
    grammar.md         ;; this file
    examples/          ;; small example .nexus files
  src/                 ;; the nexus daemon
  ARCHITECTURE.md
  flake.nix
  LICENSE.md
```
