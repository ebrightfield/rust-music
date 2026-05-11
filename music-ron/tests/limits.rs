// QA W2: dedicated coverage for MAX_INPUT_BYTES, MAX_NESTING_DEPTH, MAX_VEC_LEN.
// These limits guard against adversarial input (REQ-O38, REQ-O40).

use music_ron::{parse, MusicRonError};

/// MAX_INPUT_BYTES is 1 MiB. Inputs one byte over should be rejected before
/// ron even sees them, producing a SyntaxError that mentions the byte limit.
#[test]
fn input_just_over_max_bytes_rejected() {
    let huge = "x".repeat(1_048_577);
    let err = parse(&huge).unwrap_err();
    match err {
        MusicRonError::SyntaxError { message, .. } => {
            assert!(
                message.contains("max length") || message.contains("exceeds"),
                "expected byte-limit message, got: {message}"
            );
        }
        other => panic!("expected SyntaxError, got {other:?}"),
    }
}

/// Inputs exactly at MAX_INPUT_BYTES should be let through the preflight
/// (they'll still fail to parse as RON — a SyntaxError about a bad token, not
/// about length).
#[test]
fn input_exactly_max_bytes_passes_preflight() {
    let limit = "x".repeat(1_048_576);
    let err = parse(&limit).unwrap_err();
    match err {
        MusicRonError::SyntaxError { message, .. } => {
            assert!(
                !message.contains("max length") && !message.contains("exceeds max length"),
                "input at the limit should pass preflight, but preflight rejected it: {message}"
            );
        }
        other => panic!("expected SyntaxError, got {other:?}"),
    }
}

/// MAX_NESTING_DEPTH is 64. A deeply nested Tuplet chain should be rejected
/// by ron's recursion limit (REQ-O40) before the stack overflows.
#[test]
fn deeply_nested_tuplet_rejected() {
    // Build a Snippet whose events contain a tuplet nested 200 levels deep.
    let depth = 200;
    let mut nested = String::from("[]");
    for _ in 0..depth {
        nested = format!(
            "[Tuplet(numerator: 3, denominator: 2, base: Qtr, children: {})]",
            nested
        );
    }
    let src = format!(r#"(kind: "Snippet", clef: "treble", events: {})"#, nested);

    let err = parse(&src).unwrap_err();
    assert!(
        matches!(err, MusicRonError::SyntaxError { .. }),
        "expected SyntaxError for deep nesting, got {err:?}"
    );
}

/// Shallow nesting (well under the limit) should parse fine.
#[test]
fn moderately_nested_tuplet_accepted() {
    let depth = 10;
    let mut nested = String::from("[]");
    for _ in 0..depth {
        nested = format!(
            "[Tuplet(numerator: 3, denominator: 2, base: Qtr, children: {})]",
            nested
        );
    }
    let src = format!(r#"(kind: "Snippet", clef: "treble", events: {})"#, nested);
    parse(&src).unwrap_or_else(|e| panic!("shallow nesting should parse, got {e}"));
}

/// MAX_VEC_LEN is 10_000. A snippet with >10k events must be rejected by the
/// bounded-vec visitor wired into Snippet.events (W1 fix).
#[test]
fn events_vec_over_max_len_rejected() {
    let count = 10_001;
    let mut events = String::from("[");
    for i in 0..count {
        if i > 0 {
            events.push(',');
        }
        events.push_str("Rest(duration: \"4\")");
    }
    events.push(']');
    let src = format!(r#"(kind: "Snippet", clef: "treble", events: {})"#, events);

    let err = parse(&src).unwrap_err();
    assert!(
        matches!(err, MusicRonError::SyntaxError { .. }),
        "expected SyntaxError for >MAX_VEC_LEN events, got {err:?}"
    );
}
