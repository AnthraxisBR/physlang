//! Parser tests for simulate declarations

use physlang_core::parse_program;

#[test]
fn test_simulate_basic() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let program = result.unwrap();
    assert_eq!(program.simulate.dt, 0.01);
    assert_eq!(program.simulate.steps, 100);
}

#[test]
fn test_simulate_large_steps() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.001 steps = 10000
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert_eq!(program.simulate.dt, 0.001);
    assert_eq!(program.simulate.steps, 10000);
}

#[test]
fn test_simulate_small_dt() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.0001 steps = 1
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert!((program.simulate.dt - 0.0001).abs() < 1e-6);
    assert_eq!(program.simulate.steps, 1);
}

#[test]
fn test_simulate_zero_steps() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 0
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert_eq!(program.simulate.steps, 0);
}

#[test]
fn test_simulate_missing_block() {
    let source = "particle a at (0.0, 0.0) mass 1.0";
    let result = parse_program(source);
    assert!(result.is_err(), "Should fail without simulate block");
}

