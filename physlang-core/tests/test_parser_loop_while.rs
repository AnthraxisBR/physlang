//! Parser tests for while-loop declarations

use physlang_core::parse_program;

#[test]
fn test_while_loop_position_x_less_than() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop while position(a).x < 5.0 with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.5 direction (1.0, 0.0)
}
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let program = result.unwrap();
    assert_eq!(program.loops.len(), 1);
    match &program.loops[0].kind {
        physlang_core::ast::LoopKind::WhileCondition { condition, frequency, damping, target } => {
            assert_eq!(*frequency, 1.0);
            assert_eq!(*damping, 0.0);
            assert_eq!(target, "a");
            match condition {
                physlang_core::ast::ConditionExpr::LessThan(obs, threshold) => {
                    assert_eq!(*threshold, 5.0);
                    match obs {
                        physlang_core::ast::ObservableExpr::PositionX(name) => {
                            assert_eq!(name, "a");
                        }
                        _ => panic!("Expected PositionX observable"),
                    }
                }
                _ => panic!("Expected LessThan condition"),
            }
        }
        _ => panic!("Expected WhileCondition loop"),
    }
}

#[test]
fn test_while_loop_position_x_greater_than() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop while position(a).x > 10.0 with frequency 2.0 damping 0.05 on a {
    force push(a) magnitude 1.0 direction (-1.0, 0.0)
}
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    match &program.loops[0].kind {
        physlang_core::ast::LoopKind::WhileCondition { condition, .. } => {
            match condition {
                physlang_core::ast::ConditionExpr::GreaterThan(obs, threshold) => {
                    assert_eq!(*threshold, 10.0);
                    match obs {
                        physlang_core::ast::ObservableExpr::PositionX(name) => {
                            assert_eq!(name, "a");
                        }
                        _ => panic!("Expected PositionX observable"),
                    }
                }
                _ => panic!("Expected GreaterThan condition"),
            }
        }
        _ => panic!("Expected WhileCondition loop"),
    }
}

#[test]
fn test_while_loop_position_y() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop while position(a).y < 3.0 with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.5 direction (0.0, 1.0)
}
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    match &program.loops[0].kind {
        physlang_core::ast::LoopKind::WhileCondition { condition, .. } => {
            match condition {
                physlang_core::ast::ConditionExpr::LessThan(obs, threshold) => {
                    assert_eq!(*threshold, 3.0);
                    match obs {
                        physlang_core::ast::ObservableExpr::PositionY(name) => {
                            assert_eq!(name, "a");
                        }
                        _ => panic!("Expected PositionY observable"),
                    }
                }
                _ => panic!("Expected LessThan condition"),
            }
        }
        _ => panic!("Expected WhileCondition loop"),
    }
}

#[test]
fn test_while_loop_distance() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
loop while distance(a, b) > 2.0 with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.5 direction (1.0, 0.0)
}
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    match &program.loops[0].kind {
        physlang_core::ast::LoopKind::WhileCondition { condition, .. } => {
            match condition {
                physlang_core::ast::ConditionExpr::GreaterThan(obs, threshold) => {
                    assert_eq!(*threshold, 2.0);
                    match obs {
                        physlang_core::ast::ObservableExpr::Distance(a, b) => {
                            assert_eq!(a, "a");
                            assert_eq!(b, "b");
                        }
                        _ => panic!("Expected Distance observable"),
                    }
                }
                _ => panic!("Expected GreaterThan condition"),
            }
        }
        _ => panic!("Expected WhileCondition loop"),
    }
}

