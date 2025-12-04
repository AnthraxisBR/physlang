//! Parser tests for particle declarations

use physlang_core::parse_program;

#[test]
fn test_single_particle() {
    let source = "particle a at (1.0, 2.0) mass 3.0\nsimulate dt = 0.01 steps = 100";
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let program = result.unwrap();
    assert_eq!(program.particles.len(), 1);
    assert_eq!(program.particles[0].name, "a");
    assert_eq!(program.particles[0].position.x, 1.0);
    assert_eq!(program.particles[0].position.y, 2.0);
    assert_eq!(program.particles[0].mass, 3.0);
}

#[test]
fn test_multiple_particles() {
    let source = r#"
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 10.0) mass 2.5
particle c at (-1.5, 3.14) mass 0.1
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let program = result.unwrap();
    assert_eq!(program.particles.len(), 3);
    assert_eq!(program.particles[0].name, "a");
    assert_eq!(program.particles[1].name, "b");
    assert_eq!(program.particles[2].name, "c");
    assert_eq!(program.particles[1].position.x, 5.0);
    assert_eq!(program.particles[1].position.y, 10.0);
    assert_eq!(program.particles[2].mass, 0.1);
}

#[test]
fn test_particle_negative_coordinates() {
    let source = "particle neg at (-10.5, -20.3) mass 1.0\nsimulate dt = 0.01 steps = 100";
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert_eq!(program.particles[0].position.x, -10.5);
    assert_eq!(program.particles[0].position.y, -20.3);
}

#[test]
fn test_particle_zero_mass() {
    let source = "particle zero at (0.0, 0.0) mass 0.0\nsimulate dt = 0.01 steps = 100";
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert_eq!(program.particles[0].mass, 0.0);
}

#[test]
fn test_particle_with_comment() {
    let source = r#"
# This is a comment
particle a at (1.0, 2.0) mass 3.0
simulate dt = 0.01 steps = 100
"#;
    let result = parse_program(source);
    assert!(result.is_ok());
    
    let program = result.unwrap();
    assert_eq!(program.particles.len(), 1);
}

#[test]
fn test_particle_missing_simulate() {
    let source = "particle a at (1.0, 2.0) mass 3.0";
    let result = parse_program(source);
    // Parser should require simulate block
    assert!(result.is_err());
}

