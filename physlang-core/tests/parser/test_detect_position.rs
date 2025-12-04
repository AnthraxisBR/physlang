//! Parser tests for position detector declarations

use physlang_core::parse_program;

#[test]
fn test_position_detector() {
    let source = r#"
particle a at (1.0, 2.0) mass 1.0
simulate dt = 0.01 steps = 100
detect a_pos = position(a)
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let program = result.unwrap();
    assert_eq!(program.detectors.len(), 1);
    assert_eq!(program.detectors[0].name, "a_pos");
    match &program.detectors[0].kind {
        physlang_core::ast::DetectorKind::Position(name) => {
            assert_eq!(name, "a");
        }
        _ => panic!("Expected position detector"),
    }
}

#[test]
fn test_multiple_position_detectors() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
simulate dt = 0.01 steps = 100
detect pos_a = position(a)
detect pos_b = position(b)
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert_eq!(program.detectors.len(), 2);
    assert_eq!(program.detectors[0].name, "pos_a");
    assert_eq!(program.detectors[1].name, "pos_b");
}

#[test]
fn test_position_detector_with_underscore_name() {
    let source = r#"
particle my_particle at (1.0, 2.0) mass 1.0
simulate dt = 0.01 steps = 100
detect my_detector = position(my_particle)
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    match &program.detectors[0].kind {
        physlang_core::ast::DetectorKind::Position(name) => {
            assert_eq!(name, "my_particle");
        }
        _ => panic!("Expected position detector"),
    }
}

