//! Analyzer tests for valid programs (should have no errors)

use physlang_core::{analyze_program, parse_program};

#[test]
fn test_valid_simple_program() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
force gravity(a, b) G = 1.0
force spring(a, b) k = 5.0 rest = 3.0
simulate dt = 0.01 steps = 100
detect pos_a = position(a)
detect dist_ab = distance(a, b)
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(!diagnostics.has_errors(), "Valid program should have no errors: {:?}", diagnostics.errors().collect::<Vec<_>>());
}

#[test]
fn test_valid_program_with_loop() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop for 5 cycles with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 1.0 direction (1.0, 0.0)
}
detect pos_a = position(a)
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(!diagnostics.has_errors());
}

#[test]
fn test_valid_program_with_well() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
well target on a if position(a).x >= 5.0 depth 10.0
simulate dt = 0.01 steps = 100
detect pos_a = position(a)
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(!diagnostics.has_errors());
}

#[test]
fn test_valid_program_with_while_loop() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop while position(a).x < 5.0 with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.5 direction (1.0, 0.0)
}
detect pos_a = position(a)
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(!diagnostics.has_errors());
}

#[test]
fn test_valid_program_complex() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
force gravity(a, b) G = 1.0
force spring(a, b) k = 5.0 rest = 3.0
well target on a if position(a).x >= 5.0 depth 10.0
simulate dt = 0.01 steps = 100
loop for 10 cycles with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.5 direction (1.0, 0.0)
}
loop while distance(a, b) > 2.0 with frequency 1.0 damping 0.0 on b {
    force push(b) magnitude 0.3 direction (-1.0, 0.0)
}
detect pos_a = position(a)
detect dist_ab = distance(a, b)
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(!diagnostics.has_errors());
}

