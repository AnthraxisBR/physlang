//! Analyzer tests for duplicate particle names

use physlang_core::{analyze_program, parse_program};

#[test]
fn test_duplicate_particle_name() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle a at (5.0, 0.0) mass 2.0
simulate dt = 0.01 steps = 100
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(diagnostics.has_errors());
    let errors: Vec<_> = diagnostics.errors().collect();
    assert!(errors.iter().any(|e| e.message.contains("duplicate particle name 'a'")));
}

#[test]
fn test_unique_particle_names() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 2.0
particle c at (0.0, 5.0) mass 3.0
simulate dt = 0.01 steps = 100
"#;
    let program = parse_program(source).unwrap();
    let diagnostics = analyze_program(&program);
    assert!(!diagnostics.has_errors());
}

