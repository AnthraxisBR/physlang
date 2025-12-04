//! Integration tests for PhysLang programs

use physlang_core::tests::test_helpers::{run_phys_file, approx_eq_f32, results_approx_equal};
use std::path::PathBuf;

fn test_data_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("integration");
    path.push("data");
    path.push(filename);
    path
}

#[test]
fn test_simple_gravity() {
    let path = test_data_path("simple_gravity.phys");
    let result = run_phys_file(path.to_str().unwrap()).expect("Failed to run program");
    
    // Should have 2 detectors
    assert_eq!(result.detectors.len(), 2);
    
    // Find detectors by name
    let pos_a = result.detectors.iter().find(|d| d.name == "pos_a").unwrap();
    let dist_ab = result.detectors.iter().find(|d| d.name == "dist_ab").unwrap();
    
    // After 100 steps with gravity, particles should have moved closer
    // Distance should be less than initial 5.0
    assert!(dist_ab.value < 5.0, "Distance should decrease due to gravity");
    assert!(dist_ab.value > 0.0, "Distance should be positive");
}

#[test]
fn test_spring_oscillator() {
    let path = test_data_path("spring_oscillator.phys");
    let result = run_phys_file(path.to_str().unwrap()).expect("Failed to run program");
    
    // Spring should oscillate around rest length
    let dist_ab = result.detectors.iter().find(|d| d.name == "dist_ab").unwrap();
    
    // After many steps, should be close to rest length (2.0) or oscillating
    // Allow some tolerance for oscillation
    assert!((dist_ab.value - 2.0).abs() < 1.5, 
        "Distance should be near rest length, got {}", dist_ab.value);
}

#[test]
fn test_loop_push() {
    let path = test_data_path("loop_push.phys");
    let result = run_phys_file(path.to_str().unwrap()).expect("Failed to run program");
    
    let a_x = result.detectors.iter().find(|d| d.name == "a_x").unwrap();
    
    // Loop pushes 5 times with magnitude 0.5 each
    // After pushes, particle should have moved right
    assert!(a_x.value > 0.0, "Particle should move right after pushes");
    // Exact value depends on timing, but should be positive
}

#[test]
fn test_while_loop_stop() {
    let path = test_data_path("while_loop_stop.phys");
    let result = run_phys_file(path.to_str().unwrap()).expect("Failed to run program");
    
    let a_x = result.detectors.iter().find(|d| d.name == "a_x").unwrap();
    
    // Loop should stop when position.x >= 3.0
    // Final position should be around 3.0 or slightly above
    assert!(a_x.value >= 2.5, "Particle should reach threshold, got {}", a_x.value);
    assert!(a_x.value <= 4.0, "Particle should not overshoot too much, got {}", a_x.value);
}

#[test]
fn test_graph_layout() {
    let path = test_data_path("graph_layout.phys");
    let result = run_phys_file(path.to_str().unwrap()).expect("Failed to run program");
    
    // Graph should have settled into a layout
    let dist_12 = result.detectors.iter().find(|d| d.name == "dist_12").unwrap();
    
    // Distance should be close to rest length (1.0) with some tolerance
    assert!((dist_12.value - 1.0).abs() < 0.5, 
        "Distance should be near rest length, got {}", dist_12.value);
}

#[test]
fn test_integration_tolerance() {
    // Test that integration tests use tolerance correctly
    let path = test_data_path("simple_gravity.phys");
    let result1 = run_phys_file(path.to_str().unwrap()).expect("Failed to run program");
    let result2 = run_phys_file(path.to_str().unwrap()).expect("Failed to run program");
    
    // Results should be approximately equal (within tolerance)
    assert!(results_approx_equal(&result1, &result2, 1e-4),
        "Running same program twice should give similar results");
}

