//! Parser tests for gravity force declarations

use physlang_core::parse_program;

#[test]
fn test_gravity_force() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
force gravity(a, b) G = 9.81
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let program = result.unwrap();
    assert_eq!(program.forces.len(), 1);
    match &program.forces[0].kind {
        physlang_core::ast::ForceKind::Gravity { g } => {
            assert_eq!(*g, 9.81);
        }
        _ => panic!("Expected gravity force"),
    }
    assert_eq!(program.forces[0].a, "a");
    assert_eq!(program.forces[0].b, "b");
}

#[test]
fn test_gravity_force_negative_g() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
force gravity(a, b) G = -1.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    match &program.forces[0].kind {
        physlang_core::ast::ForceKind::Gravity { g } => {
            assert_eq!(*g, -1.0);
        }
        _ => panic!("Expected gravity force"),
    }
}

#[test]
fn test_gravity_force_small_g() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
force gravity(a, b) G = 0.0001
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    match &program.forces[0].kind {
        physlang_core::ast::ForceKind::Gravity { g } => {
            assert!((*g - 0.0001).abs() < 1e-6);
        }
        _ => panic!("Expected gravity force"),
    }
}

#[test]
fn test_multiple_gravity_forces() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
particle c at (0.0, 5.0) mass 1.0
force gravity(a, b) G = 1.0
force gravity(a, c) G = 2.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert_eq!(program.forces.len(), 2);
    match &program.forces[0].kind {
        physlang_core::ast::ForceKind::Gravity { g } => assert_eq!(*g, 1.0),
        _ => panic!("Expected gravity force"),
    }
    match &program.forces[1].kind {
        physlang_core::ast::ForceKind::Gravity { g } => assert_eq!(*g, 2.0),
        _ => panic!("Expected gravity force"),
    }
}

