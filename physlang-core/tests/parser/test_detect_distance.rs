//! Parser tests for distance detector declarations

use physlang_core::parse_program;

#[test]
fn test_distance_detector() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (3.0, 4.0) mass 1.0
simulate dt = 0.01 steps = 100
detect dist_ab = distance(a, b)
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let program = result.unwrap();
    assert_eq!(program.detectors.len(), 1);
    assert_eq!(program.detectors[0].name, "dist_ab");
    match &program.detectors[0].kind {
        physlang_core::ast::DetectorKind::Distance { a, b } => {
            assert_eq!(a, "a");
            assert_eq!(b, "b");
        }
        _ => panic!("Expected distance detector"),
    }
}

#[test]
fn test_multiple_distance_detectors() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (1.0, 0.0) mass 1.0
particle c at (0.0, 1.0) mass 1.0
simulate dt = 0.01 steps = 100
detect dist_ab = distance(a, b)
detect dist_ac = distance(a, c)
detect dist_bc = distance(b, c)
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert_eq!(program.detectors.len(), 3);
    match &program.detectors[0].kind {
        physlang_core::ast::DetectorKind::Distance { a, b } => {
            assert_eq!(a, "a");
            assert_eq!(b, "b");
        }
        _ => panic!("Expected distance detector"),
    }
}

#[test]
fn test_distance_and_position_detectors_together() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
detect pos_a = position(a)
detect dist_ab = distance(a, b)
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert_eq!(program.detectors.len(), 2);
    match &program.detectors[0].kind {
        physlang_core::ast::DetectorKind::Position(_) => {}
        _ => panic!("First detector should be position"),
    }
    match &program.detectors[1].kind {
        physlang_core::ast::DetectorKind::Distance { .. } => {}
        _ => panic!("Second detector should be distance"),
    }
}

