//! Analyzer tests for unknown particle references

use physlang_core::{analyze_program, parse_program};

#[test]
fn test_unknown_particle_in_force() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
force gravity(a, b) G = 1.0
simulate dt = 0.01 steps = 100
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(diagnostics.has_errors());
    let errors: Vec<_> = diagnostics.errors().collect();
    assert!(errors.iter().any(|e| e.message.contains("unknown particle 'b'")));
}

#[test]
fn test_unknown_particle_in_loop_target() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop for 5 cycles with frequency 1.0 damping 0.0 on b {
    force push(a) magnitude 1.0 direction (1.0, 0.0)
}
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(diagnostics.has_errors());
    let errors: Vec<_> = diagnostics.errors().collect();
    assert!(errors.iter().any(|e| e.message.contains("unknown particle 'b'") && e.message.contains("loop target")));
}

#[test]
fn test_unknown_particle_in_loop_body_push() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop for 5 cycles with frequency 1.0 damping 0.0 on a {
    force push(b) magnitude 1.0 direction (1.0, 0.0)
}
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(diagnostics.has_errors());
    let errors: Vec<_> = diagnostics.errors().collect();
    assert!(errors.iter().any(|e| e.message.contains("unknown particle 'b'") && e.message.contains("loop body push")));
}

#[test]
fn test_unknown_particle_in_detector() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
detect pos_b = position(b)
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(diagnostics.has_errors());
    let errors: Vec<_> = diagnostics.errors().collect();
    assert!(errors.iter().any(|e| e.message.contains("unknown particle 'b'") && e.message.contains("detector")));
}

#[test]
fn test_unknown_particle_in_distance_detector() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
detect dist = distance(a, c)
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(diagnostics.has_errors());
    let errors: Vec<_> = diagnostics.errors().collect();
    assert!(errors.iter().any(|e| e.message.contains("unknown particle 'c'")));
}

#[test]
fn test_unknown_particle_in_well() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
well w on b if position(a).x >= 5.0 depth 10.0
simulate dt = 0.01 steps = 100
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(diagnostics.has_errors());
    let errors: Vec<_> = diagnostics.errors().collect();
    assert!(errors.iter().any(|e| e.message.contains("unknown particle 'b'") && e.message.contains("well")));
}

#[test]
fn test_unknown_particle_in_well_observable() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
well w on a if position(b).x >= 5.0 depth 10.0
simulate dt = 0.01 steps = 100
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(diagnostics.has_errors());
    let errors: Vec<_> = diagnostics.errors().collect();
    assert!(errors.iter().any(|e| e.message.contains("unknown particle 'b'") && e.message.contains("observable")));
}

#[test]
fn test_unknown_particle_in_while_condition() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop while position(b).x < 5.0 with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 1.0 direction (1.0, 0.0)
}
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(diagnostics.has_errors());
    let errors: Vec<_> = diagnostics.errors().collect();
    assert!(errors.iter().any(|e| e.message.contains("unknown particle 'b'")));
}

#[test]
fn test_unknown_particle_in_distance_condition() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop while distance(a, c) > 2.0 with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 1.0 direction (1.0, 0.0)
}
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(diagnostics.has_errors());
    let errors: Vec<_> = diagnostics.errors().collect();
    assert!(errors.iter().any(|e| e.message.contains("unknown particle 'c'")));
}

