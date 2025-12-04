//! Parser tests for spring force declarations

use physlang_core::parse_program;

#[test]
fn test_spring_force() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
force spring(a, b) k = 10.0 rest = 3.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let program = result.unwrap();
    assert_eq!(program.forces.len(), 1);
    match &program.forces[0].kind {
        physlang_core::ast::ForceKind::Spring { k, rest } => {
            assert_eq!(*k, 10.0);
            assert_eq!(*rest, 3.0);
        }
        _ => panic!("Expected spring force"),
    }
    assert_eq!(program.forces[0].a, "a");
    assert_eq!(program.forces[0].b, "b");
}

#[test]
fn test_spring_force_zero_rest() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
force spring(a, b) k = 5.0 rest = 0.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    match &program.forces[0].kind {
        physlang_core::ast::ForceKind::Spring { rest, .. } => {
            assert_eq!(*rest, 0.0);
        }
        _ => panic!("Expected spring force"),
    }
}

#[test]
fn test_spring_force_negative_k() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
force spring(a, b) k = -1.0 rest = 2.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    match &program.forces[0].kind {
        physlang_core::ast::ForceKind::Spring { k, .. } => {
            assert_eq!(*k, -1.0);
        }
        _ => panic!("Expected spring force"),
    }
}

#[test]
fn test_spring_force_large_rest() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
force spring(a, b) k = 1.0 rest = 100.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    match &program.forces[0].kind {
        physlang_core::ast::ForceKind::Spring { rest, .. } => {
            assert_eq!(*rest, 100.0);
        }
        _ => panic!("Expected spring force"),
    }
}

#[test]
fn test_spring_and_gravity_together() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
force gravity(a, b) G = 1.0
force spring(a, b) k = 5.0 rest = 3.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert_eq!(program.forces.len(), 2);
    match &program.forces[0].kind {
        physlang_core::ast::ForceKind::Gravity { .. } => {}
        _ => panic!("First force should be gravity"),
    }
    match &program.forces[1].kind {
        physlang_core::ast::ForceKind::Spring { .. } => {}
        _ => panic!("Second force should be spring"),
    }
}

