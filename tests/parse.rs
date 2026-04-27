//! Integration tests for `nexus::QueryParser`.
//!
//! Covers all four query kinds (Node / Edge / Graph / KindDecl),
//! all three PatternField variants per kind, and the negative
//! cases: wrong delimiter, lowercase head, wrong bind name,
//! uppercase bind name, unknown kind, unknown RelationKind
//! variant, trailing tokens, unclosed query.

use nexus::{Error, QueryParser};
use signal::{
    EdgeQuery, GraphQuery, KindDeclQuery, NodeQuery, PatternField, QueryOp, RelationKind, Slot,
};

// ─── Node queries ─────────────────────────────────────────────

#[test]
fn node_query_with_bind() {
    let parsed = QueryParser::new("(| Node @name |)").into_query().unwrap();
    assert_eq!(
        parsed,
        QueryOp::Node(NodeQuery {
            name: PatternField::Bind,
        })
    );
}

#[test]
fn node_query_with_wildcard() {
    let parsed = QueryParser::new("(| Node _ |)").into_query().unwrap();
    assert_eq!(
        parsed,
        QueryOp::Node(NodeQuery {
            name: PatternField::Wildcard,
        })
    );
}

#[test]
fn node_query_with_quoted_string_match() {
    let parsed = QueryParser::new(r#"(| Node "Alice" |)"#).into_query().unwrap();
    assert_eq!(
        parsed,
        QueryOp::Node(NodeQuery {
            name: PatternField::Match("Alice".to_string()),
        })
    );
}

#[test]
fn node_query_with_bare_identifier_match() {
    let parsed = QueryParser::new("(| Node Alice |)").into_query().unwrap();
    assert_eq!(
        parsed,
        QueryOp::Node(NodeQuery {
            name: PatternField::Match("Alice".to_string()),
        })
    );
}

#[test]
fn node_query_with_kebab_case_match() {
    let parsed = QueryParser::new("(| Node nexus-daemon |)").into_query().unwrap();
    assert_eq!(
        parsed,
        QueryOp::Node(NodeQuery {
            name: PatternField::Match("nexus-daemon".to_string()),
        })
    );
}

// ─── Edge queries ─────────────────────────────────────────────

#[test]
fn edge_query_all_wildcards() {
    let parsed = QueryParser::new("(| Edge _ _ _ |)").into_query().unwrap();
    assert_eq!(
        parsed,
        QueryOp::Edge(EdgeQuery {
            from: PatternField::Wildcard,
            to: PatternField::Wildcard,
            kind: PatternField::Wildcard,
        })
    );
}

#[test]
fn edge_query_all_binds() {
    let parsed = QueryParser::new("(| Edge @from @to @kind |)").into_query().unwrap();
    assert_eq!(
        parsed,
        QueryOp::Edge(EdgeQuery {
            from: PatternField::Bind,
            to: PatternField::Bind,
            kind: PatternField::Bind,
        })
    );
}

#[test]
fn edge_query_mixed_match_bind_wildcard() {
    // (| Edge 100 @to DependsOn |)
    // exercises all three PatternField variants in one query.
    let parsed = QueryParser::new("(| Edge 100 @to DependsOn |)").into_query().unwrap();
    assert_eq!(
        parsed,
        QueryOp::Edge(EdgeQuery {
            from: PatternField::Match(Slot(100)),
            to: PatternField::Bind,
            kind: PatternField::Match(RelationKind::DependsOn),
        })
    );
}

#[test]
fn edge_query_match_all_relation_kind_variants() {
    // Spot-check each RelationKind variant parses.
    for variant_name in [
        "Flow",
        "DependsOn",
        "Contains",
        "References",
        "Produces",
        "Consumes",
        "Calls",
        "Implements",
        "IsA",
    ] {
        let input = format!("(| Edge _ _ {variant_name} |)");
        let parsed = QueryParser::new(&input).into_query().unwrap();
        match parsed {
            QueryOp::Edge(EdgeQuery {
                kind: PatternField::Match(_),
                ..
            }) => {}
            other => panic!("expected Edge with Match kind for {variant_name}, got {other:?}"),
        }
    }
}

// ─── Graph queries ────────────────────────────────────────────

#[test]
fn graph_query_with_quoted_title() {
    let parsed = QueryParser::new(r#"(| Graph "criome request flow" |)"#)
        .into_query()
        .unwrap();
    assert_eq!(
        parsed,
        QueryOp::Graph(GraphQuery {
            title: PatternField::Match("criome request flow".to_string()),
        })
    );
}

#[test]
fn graph_query_with_bind() {
    let parsed = QueryParser::new("(| Graph @title |)").into_query().unwrap();
    assert_eq!(
        parsed,
        QueryOp::Graph(GraphQuery {
            title: PatternField::Bind,
        })
    );
}

// ─── KindDecl queries ─────────────────────────────────────────

#[test]
fn kind_decl_query_with_match() {
    let parsed = QueryParser::new("(| KindDecl Node |)").into_query().unwrap();
    assert_eq!(
        parsed,
        QueryOp::KindDecl(KindDeclQuery {
            name: PatternField::Match("Node".to_string()),
        })
    );
}

#[test]
fn kind_decl_query_with_wildcard() {
    let parsed = QueryParser::new("(| KindDecl _ |)").into_query().unwrap();
    assert_eq!(
        parsed,
        QueryOp::KindDecl(KindDeclQuery {
            name: PatternField::Wildcard,
        })
    );
}

// ─── Bind name validation ────────────────────────────────────

#[test]
fn wrong_bind_name_for_node_rejected() {
    // Node's only field is `name`; @anything-else should fail.
    let error = QueryParser::new("(| Node @wrongname |)")
        .into_query()
        .unwrap_err();
    match error {
        Error::WrongBindName {
            expected_field_name,
            got_bind_name,
        } => {
            assert_eq!(expected_field_name, "name");
            assert_eq!(got_bind_name, "wrongname");
        }
        other => panic!("expected WrongBindName, got {other:?}"),
    }
}

#[test]
fn wrong_bind_name_for_edge_first_position_rejected() {
    // Edge's first field is `from`; @to in position 1 should fail.
    let error = QueryParser::new("(| Edge @to _ _ |)").into_query().unwrap_err();
    match error {
        Error::WrongBindName {
            expected_field_name,
            got_bind_name,
        } => {
            assert_eq!(expected_field_name, "from");
            assert_eq!(got_bind_name, "to");
        }
        other => panic!("expected WrongBindName, got {other:?}"),
    }
}

#[test]
fn wrong_bind_name_for_edge_second_position_rejected() {
    // Edge.to field; @from in position 2 should fail.
    let error = QueryParser::new("(| Edge _ @from _ |)").into_query().unwrap_err();
    match error {
        Error::WrongBindName {
            expected_field_name,
            got_bind_name,
        } => {
            assert_eq!(expected_field_name, "to");
            assert_eq!(got_bind_name, "from");
        }
        other => panic!("expected WrongBindName, got {other:?}"),
    }
}

#[test]
fn pascal_case_bind_name_rejected() {
    // @From would be PascalCase — bind names must be lowercase.
    let error = QueryParser::new("(| Edge @From _ _ |)").into_query().unwrap_err();
    assert!(
        matches!(error, Error::ExpectedLowercaseIdentifier { .. }),
        "expected ExpectedLowercaseIdentifier, got {error:?}"
    );
}

// ─── Other parse errors ───────────────────────────────────────

#[test]
fn lowercase_kind_head_rejected() {
    // (| node ... |) — head must be PascalCase.
    let error = QueryParser::new("(| node @name |)").into_query().unwrap_err();
    assert!(
        matches!(error, Error::ExpectedPascalIdentifier { .. }),
        "expected ExpectedPascalIdentifier, got {error:?}"
    );
}

#[test]
fn unknown_kind_rejected() {
    let error = QueryParser::new("(| Frobnicator @x |)").into_query().unwrap_err();
    match error {
        Error::UnknownQueryKind { kind_name } => {
            assert_eq!(kind_name, "Frobnicator");
        }
        other => panic!("expected UnknownQueryKind, got {other:?}"),
    }
}

#[test]
fn unknown_relation_kind_variant_rejected() {
    // PascalCase but not a real RelationKind variant.
    let error = QueryParser::new("(| Edge _ _ Frobnicates |)")
        .into_query()
        .unwrap_err();
    match error {
        Error::UnknownRelationKindVariant { got, .. } => {
            assert_eq!(got, "Frobnicates");
        }
        other => panic!("expected UnknownRelationKindVariant, got {other:?}"),
    }
}

#[test]
fn wrong_outer_delimiter_rejected() {
    // (Node ...) is an Assert; not a query. parse_query expects (|.
    let error = QueryParser::new("(Node Alice)").into_query().unwrap_err();
    assert!(
        matches!(error, Error::UnexpectedToken { .. }),
        "expected UnexpectedToken, got {error:?}"
    );
}

#[test]
fn unclosed_query_rejected() {
    let error = QueryParser::new("(| Node @name").into_query().unwrap_err();
    assert!(
        matches!(error, Error::UnexpectedToken { .. }),
        "expected UnexpectedToken (looking for |)), got {error:?}"
    );
}

#[test]
fn trailing_tokens_after_query_rejected() {
    let error = QueryParser::new("(| Node @name |) extra")
        .into_query()
        .unwrap_err();
    assert!(
        matches!(error, Error::TrailingTokens { .. }),
        "expected TrailingTokens, got {error:?}"
    );
}

// ─── Streaming via parse() ────────────────────────────────────

#[test]
fn streaming_two_queries_in_a_row() {
    // The streaming API should yield both queries from one parser.
    let mut parser = QueryParser::new("(| Node @name |) (| Edge @from @to @kind |)");

    let first = parser.parse().unwrap();
    assert_eq!(
        first,
        QueryOp::Node(NodeQuery {
            name: PatternField::Bind,
        })
    );

    let second = parser.parse().unwrap();
    assert_eq!(
        second,
        QueryOp::Edge(EdgeQuery {
            from: PatternField::Bind,
            to: PatternField::Bind,
            kind: PatternField::Bind,
        })
    );
}
