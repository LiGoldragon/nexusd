# nexus

Nexus is a typed text format for Sema requests and replies. It mirrors rkyv in
text: records and sequences carry the structure; Rust schema types carry the
meaning.

The core rule:

> Delimiters define structure. Head identifiers define meaning. The receiving
> type decides how a field decodes.

Nexus is not an expression language. It has no operators, no function calls,
and no special delimiter for query forms. A top-level message is a typed record,
usually a variant of the closed `Request` or `Reply` enum.

---

## 1 · Grammar Surface

The Tier 0 grammar has two delimiter pairs and two in-position pattern markers.

| Construct | Form | Role |
|---|---|---|
| Record | `(Kind field0 field1 ...)` | Named positional composite |
| Sequence | `[elem0 elem1 ...]` | Ordered collection |
| Bind | `@fieldname` | Pattern bind in a `PatternField<T>` position |
| Wildcard | `_` | Pattern wildcard in a `PatternField<T>` position |
| Path | `Name:Nested` | Nested name separator |
| Comment | `;; ...` | Line comment discarded by the parser |
| Byte literal | `#a1b2c3` | Lowercase even-length hex bytes |
| String | `"text"` | Inline quoted string |
| Multiline string | `"""..."""` | Multiline string |
| Bare string | `bare-ident` | String when the schema expects `String` |

The lexer token vocabulary is locked at the structural minimum:

```rust
pub enum Token {
    LParen,
    RParen,
    LBracket,
    RBracket,
    At,
    Colon,
    Ident(String),
    Bool(bool),
    Int(i128),
    UInt(u128),
    Float(f64),
    Str(String),
    Bytes(Vec<u8>),
}
```

The wildcard `_` is an identifier token. It becomes a wildcard only when the
receiving type is `PatternField<T>`.

---

## 2 · Records

A record opens with `(`, followed by a PascalCase head identifier and positional
fields.

```nexus
(Point 3.0 4.0)
(Node User)
(Node "nexus daemon")
(Edge 100 101 Flow)
(Line (Point 0.0 0.0) (Point 10.0 10.0))
```

Field names do not appear in text. They live in the Rust schema. The receiving
type knows how many fields to read, which types they have, and what each
position means.

Record heads also encode closed enum variants:

```nexus
Flow
(Limit 10)
(Range 0 100)
```

Unit variants may render as a bare PascalCase identifier when the receiving type
expects that enum.

---

## 3 · Sequences

A sequence opens with `[` and closes with `]`.

```nexus
[(Node alice) (Node bob) (Node carol)]
[100 101 102]
[("name" 1) ("age" 2)]
[]
```

The receiving type decides whether the sequence is a `Vec<T>`, `BTreeSet<T>`,
or a map encoded as a sequence of pair records. The wire form does not carry a
separate set or map delimiter.

---

## 4 · Patterns

Patterns are schema-driven. There is no pattern delimiter.

The same text delimiter `()` is used for data records and query records. The
receiving type decides whether `@field` and `_` are allowed.

```rust
struct Node {
    name: String,
}

struct NodeQuery {
    name: PatternField<String>,
}
```

| Text | Receiving type | Meaning |
|---|---|---|
| `(Node User)` | `Node` | concrete data record |
| `(NodeQuery @name)` | `NodeQuery` | bind the `name` field |
| `(NodeQuery _)` | `NodeQuery` | wildcard match |
| `(NodeQuery User)` | `NodeQuery` | concrete field match |
| `(Node @name)` | `Node` | parse error |

`@fieldname` must match the schema field name at that position. The position
already carries identity; the marker confirms the field being bound.

`_` is valid only when the receiving type is `PatternField<T>`. Outside a
pattern position it is not a value.

---

## 5 · Requests

Every top-level request is a verb record. Tier 0 uses fully explicit request
heads; a bare top-level domain record is not an implicit assert.

```rust
pub enum Request {
    Assert(AssertOperation),
    Mutate(MutateOperation),
    Retract(RetractOperation),
    Atomic(AtomicBatch),
    Match(MatchQuery),
    Subscribe(SubscribeQuery),
    Validate(ValidateRequest),
    Aggregate(AggregateQuery),
    Project(ProjectQuery),
    Constrain(ConstrainQuery),
    Infer(InferQuery),
    Recurse(RecurseQuery),
}
```

Examples:

```nexus
(Assert (Node User))
(Assert (Edge 100 101 Flow))

(Mutate 100 (Node "renamed"))
(Retract Node 100)
(Atomic [(Assert (Node A)) (Assert (Node B))])

(Match (NodeQuery @name) Any)
(Match (EdgeQuery 100 @to Flow) (Limit 10))
(Subscribe (NodeQuery @name) ImmediateExtension Block)
(Validate (Assert (Node "dry run")))

(Aggregate (NodeQuery @name) Count)
(Project (NodeQuery @name) (Fields [name]) Any)
(Constrain [(EdgeQuery 100 @to Flow) (NodeQuery @to)] (Unify [to]) Any)
(Infer (NodeQuery @name) StandardOntology)
(Recurse (NodeQuery @name) (EdgeQuery @from @to DependsOn) Fixpoint)
```

Slot references are bare integers in slot-typed positions. The schema tells the
decoder that `100` is a `Slot<Node>`, not an ordinary `u64`.

---

## 6 · Replies

Replies are typed records or sequences of typed records.

```nexus
(Ok)
(Diagnostic Error E0042 "no binding for unknown-target")
[(Node User) (Node "nexus daemon")]
[(Ok) (Diagnostic Error E0042 "conflict on slot 100") (Ok)]
```

When a reply needs to carry the store slot beside a returned record, use a typed
pair record rather than an anonymous tuple:

```nexus
[(SlotBinding 1024 (Node User))
 (SlotBinding 1025 (Node "nexus daemon"))]
```

`SlotBinding<T>` is the textual shape for `slot + value` reply data. It is a
named type because anonymous tuples are not used at typed boundaries.

---

## 7 · Dropped Forms

These forms are not reserved. They are outside Tier 0 and should not appear in
new examples or new parser work.

| Dropped form | Replacement |
|---|---|
| `(| Node @name |)` | `(Match (NodeQuery @name) Any)` |
| `[| op1 op2 |]` | `(Atomic [op1 op2])` |
| `{ name }` | `(Project pattern (Fields [name]) cardinality)` |
| `{| pat1 pat2 |}` | `(Constrain [pat1 pat2] (Unify [name]) cardinality)` |
| `~record` | `(Mutate slot record)` |
| `!record` | `(Retract Kind slot)` |
| `?record` | `(Validate request)` |
| `*pattern` | `(Subscribe pattern mode backpressure)` |
| `@a=@b` | Deferred; use `(Unify [a b])` where a typed record owns the behavior |
| `< > <= >= !=` | Predicates are typed records |

---

## 8 · Current Daemon Status

The existing `nexus-daemon` implementation is still Criome-specific. The Tier 0
rewrite universalizes the spec first. The daemon becomes domain-parameterized
only after another concrete domain translator needs it.

