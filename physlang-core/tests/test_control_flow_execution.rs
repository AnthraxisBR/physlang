//! Execution tests for control flow statements (v0.8)

use physlang_core::runtime::run_program;

#[test]
fn test_if_condition_true() {
    let source = r#"
let risk_level = 0.7
if risk_level > 0.5 {
    particle "A" at (0.0, 0.0) mass 1.0
    particle "B" at (3.0, 0.0) mass 1.0
}
simulate dt = 0.01 steps = 1
"#;
    let result = run_program(source);
    assert!(result.is_ok(), "Failed to run: {:?}", result.err());
    // After function execution, we should have 2 particles
    // For now, just verify it runs without error
}

#[test]
fn test_if_condition_false() {
    let source = r#"
let risk_level = 0.3
if risk_level > 0.5 {
    particle "A" at (0.0, 0.0) mass 1.0
}
simulate dt = 0.01 steps = 1
"#;
    let result = run_program(source);
    assert!(result.is_ok(), "Failed to run: {:?}", result.err());
}

#[test]
fn test_if_else_branch() {
    let source = r#"
let risk_level = 0.3
if risk_level > 0.5 {
    particle "high" at (0.0, 0.0) mass 1.0
} else {
    particle "low" at (1.0, 1.0) mass 1.0
}
simulate dt = 0.01 steps = 1
"#;
    let result = run_program(source);
    assert!(result.is_ok(), "Failed to run: {:?}", result.err());
}

#[test]
fn test_for_loop_generates_particles() {
    let source = r#"
for i in 0..5 {
    let x = i * 1.5
    particle "node" at (x, 0.0) mass 1.0
}
simulate dt = 0.01 steps = 1
"#;
    let result = run_program(source);
    assert!(result.is_ok(), "Failed to run: {:?}", result.err());
}

#[test]
fn test_nested_for_loops() {
    let source = r#"
for i in 0..3 {
    for j in 0..3 {
        let x = i * 1.5
        let y = j * 1.5
        particle "node" at (x, y) mass 1.0
    }
}
simulate dt = 0.01 steps = 1
"#;
    let result = run_program(source);
    assert!(result.is_ok(), "Failed to run: {:?}", result.err());
}

#[test]
fn test_match_literal_pattern() {
    let source = r#"
let regime = 1
match regime {
    0 => {
        particle "normal" at (0.0, 0.0) mass 1.0
    }
    1 => {
        particle "stress" at (1.0, 1.0) mass 2.0
    }
}
simulate dt = 0.01 steps = 1
"#;
    let result = run_program(source);
    assert!(result.is_ok(), "Failed to run: {:?}", result.err());
}

#[test]
fn test_match_wildcard_pattern() {
    let source = r#"
let regime = 99
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
simulate dt = 0.01 steps = 1
"#;
    let result = run_program(source);
    assert!(result.is_ok(), "Failed to run: {:?}", result.err());
}

#[test]
fn test_control_flow_in_function() {
    let source = r#"
fn create_grid(size) {
    for i in 0..size {
        for j in 0..size {
            let x = i * 1.0
            let y = j * 1.0
            particle "node" at (x, y) mass 1.0
        }
    }
}

create_grid(3)
simulate dt = 0.01 steps = 1
"#;
    let result = run_program(source);
    assert!(result.is_ok(), "Failed to run: {:?}", result.err());
}

#[test]
fn test_conditional_exposure_example() {
    let source = r#"
let risk_level = 0.7

fn exposure(a, b, exposure_value) {
    let k = exposure_value / 100.0
    force spring(a, b) k = k rest = 3.5
}

particle "A" at (0.0, 0.0) mass 1.0
particle "B" at (3.0, 0.0) mass 1.0

if risk_level > 0.5 {
    exposure("A", "B", 120.0)
} else {
    exposure("A", "B", 60.0)
}

simulate dt = 0.01 steps = 1
"#;
    let result = run_program(source);
    assert!(result.is_ok(), "Failed to run: {:?}", result.err());
}

#[test]
fn test_for_loop_empty_range() {
    let source = r#"
for i in 5..3 {
    particle "node" at (i * 1.0, 0.0) mass 1.0
}
simulate dt = 0.01 steps = 1
"#;
    let result = run_program(source);
    assert!(result.is_ok(), "Failed to run: {:?}", result.err());
    // Should execute without error, but no particles created
}

#[test]
fn test_match_no_match() {
    let source = r#"
let regime = 99
match regime {
    0 => {
        particle "normal" at (0.0, 0.0) mass 1.0
    }
    1 => {
        particle "stress" at (1.0, 1.0) mass 2.0
    }
}
simulate dt = 0.01 steps = 1
"#;
    let result = run_program(source);
    assert!(result.is_ok(), "Failed to run: {:?}", result.err());
    // Should execute without error, but no particles created
}

