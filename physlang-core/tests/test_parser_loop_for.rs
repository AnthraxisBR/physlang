//! Parser tests for for-loop declarations

use physlang_core::parse_program;

#[test]
fn test_for_loop_basic() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop for 5 cycles with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 1.0 direction (1.0, 0.0)
}
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let program = result.unwrap();
    assert_eq!(program.loops.len(), 1);
    match &program.loops[0].kind {
        physlang_core::ast::LoopKind::ForCycles { cycles, frequency, damping, target } => {
            assert_eq!(*cycles, 5);
            assert_eq!(*frequency, 1.0);
            assert_eq!(*damping, 0.0);
            assert_eq!(target, "a");
        }
        _ => panic!("Expected ForCycles loop"),
    }
}

#[test]
fn test_for_loop_with_damping() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop for 10 cycles with frequency 2.5 damping 0.1 on a {
    force push(a) magnitude 0.5 direction (0.0, 1.0)
}
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    match &program.loops[0].kind {
        physlang_core::ast::LoopKind::ForCycles { cycles, frequency, damping, .. } => {
            assert_eq!(*cycles, 10);
            assert_eq!(*frequency, 2.5);
            assert_eq!(*damping, 0.1);
        }
        _ => panic!("Expected ForCycles loop"),
    }
}

#[test]
fn test_for_loop_body_push() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop for 3 cycles with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 2.0 direction (1.0, 0.0)
}
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert_eq!(program.loops[0].body.len(), 1);
    match &program.loops[0].body[0] {
        physlang_core::ast::LoopBodyStmt::ForcePush { particle, magnitude, direction } => {
            assert_eq!(particle, "a");
            assert_eq!(*magnitude, 2.0);
            assert_eq!(direction.x, 1.0);
            assert_eq!(direction.y, 0.0);
        }
    }
}

#[test]
fn test_for_loop_multiple_cycles() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop for 100 cycles with frequency 0.5 damping 0.0 on a {
    force push(a) magnitude 0.1 direction (1.0, 0.0)
}
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    match &program.loops[0].kind {
        physlang_core::ast::LoopKind::ForCycles { cycles, .. } => {
            assert_eq!(*cycles, 100);
        }
        _ => panic!("Expected ForCycles loop"),
    }
}

