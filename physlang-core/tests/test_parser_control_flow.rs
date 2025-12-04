//! Parser tests for control flow statements (v0.8)

use physlang_core::parse_program;

#[test]
fn test_parse_if_statement() {
    let source = r#"
let risk_level = 0.7
if risk_level > 0.5 {
    particle "A" at (0.0, 0.0) mass 1.0
}
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_if_else_statement() {
    let source = r#"
let risk_level = 0.3
if risk_level > 0.5 {
    particle "A" at (0.0, 0.0) mass 1.0
} else {
    particle "B" at (1.0, 1.0) mass 2.0
}
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_for_loop() {
    let source = r#"
for i in 0..5 {
    particle "node" at (i * 1.5, 0.0) mass 1.0
}
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_nested_for_loops() {
    let source = r#"
for i in 0..3 {
    for j in 0..3 {
        let x = i * 1.5
        let y = j * 1.5
        particle "node" at (x, y) mass 1.0
    }
}
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_match_statement() {
    let source = r#"
let regime = 1
match regime {
    0 => {
        particle "normal" at (0.0, 0.0) mass 1.0
    }
    1 => {
        particle "stress" at (1.0, 1.0) mass 2.0
    }
    _ => {
        particle "fallback" at (2.0, 2.0) mass 3.0
    }
}
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_match_without_wildcard() {
    let source = r#"
let regime = 0
match regime {
    0 => {
        particle "normal" at (0.0, 0.0) mass 1.0
    }
    1 => {
        particle "stress" at (1.0, 1.0) mass 2.0
    }
}
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_control_flow_in_function() {
    let source = r#"
fn create_particles(count) {
    for i in 0..count {
        particle "p" at (i * 1.0, 0.0) mass 1.0
    }
}

create_particles(5)
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_comparison_operators() {
    let source = r#"
let a = 5.0
let b = 3.0
if a > b {
    particle "gt" at (0.0, 0.0) mass 1.0
}
if a < b {
    particle "lt" at (1.0, 1.0) mass 1.0
}
if a >= b {
    particle "ge" at (2.0, 2.0) mass 1.0
}
if a <= b {
    particle "le" at (3.0, 3.0) mass 1.0
}
if a == b {
    particle "eq" at (4.0, 4.0) mass 1.0
}
if a != b {
    particle "ne" at (5.0, 5.0) mass 1.0
}
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

