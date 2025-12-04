//! Parser tests for well declarations

use physlang_core::parse_program;

#[test]
fn test_well_position_x() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
well target on a if position(a).x >= 5.0 depth 10.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let program = result.unwrap();
    assert_eq!(program.wells.len(), 1);
    assert_eq!(program.wells[0].name, "target");
    assert_eq!(program.wells[0].particle, "a");
    assert_eq!(program.wells[0].threshold, 5.0);
    assert_eq!(program.wells[0].depth, 10.0);
    match &program.wells[0].observable {
        physlang_core::ast::ObservableExpr::PositionX(name) => {
            assert_eq!(name, "a");
        }
        _ => panic!("Expected PositionX observable"),
    }
}

#[test]
fn test_well_position_y() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
well vertical on a if position(a).y >= 10.0 depth 5.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    match &program.wells[0].observable {
        physlang_core::ast::ObservableExpr::PositionY(name) => {
            assert_eq!(name, "a");
        }
        _ => panic!("Expected PositionY observable"),
    }
    assert_eq!(program.wells[0].threshold, 10.0);
}

#[test]
fn test_well_distance() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
well separation on a if distance(a, b) >= 3.0 depth 2.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    match &program.wells[0].observable {
        physlang_core::ast::ObservableExpr::Distance(a, b) => {
            assert_eq!(a, "a");
            assert_eq!(b, "b");
        }
        _ => panic!("Expected Distance observable"),
    }
}

#[test]
fn test_multiple_wells() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
well well1 on a if position(a).x >= 5.0 depth 10.0
well well2 on a if position(a).y >= 10.0 depth 5.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert_eq!(program.wells.len(), 2);
    assert_eq!(program.wells[0].name, "well1");
    assert_eq!(program.wells[1].name, "well2");
}

#[test]
fn test_well_negative_threshold() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
well negative on a if position(a).x >= -5.0 depth 1.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert_eq!(program.wells[0].threshold, -5.0);
}

