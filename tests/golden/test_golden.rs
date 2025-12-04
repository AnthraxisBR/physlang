//! Golden tests - compare outputs to expected snapshots

use physlang_core::tests::test_helpers::{run_phys_file, result_to_json, load_expected, write_expected};
use std::path::PathBuf;

fn golden_data_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("golden");
    path.push(filename);
    path
}

fn expected_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("golden");
    path.push(filename);
    path
}

#[test]
fn test_simple_particle_golden() {
    let phys_path = golden_data_path("simple_particle.phys");
    let expected_path = expected_path("simple_particle.expected");
    
    let result = run_phys_file(phys_path.to_str().unwrap())
        .expect("Failed to run simple_particle.phys");
    
    let actual_json = result_to_json(&result);
    
    // Try to load expected, or write it if it doesn't exist
    match load_expected(expected_path.to_str().unwrap()) {
        Ok(expected_json) => {
            // Compare JSON (allowing for floating point differences)
            assert_eq!(actual_json, expected_json, 
                "Output does not match expected snapshot. If this is intentional, update the .expected file.");
        }
        Err(_) => {
            // First run - write expected file
            eprintln!("Writing expected file for first time: {:?}", expected_path);
            write_expected(expected_path.to_str().unwrap(), &actual_json)
                .expect("Failed to write expected file");
        }
    }
}

#[test]
fn test_oscillator_golden() {
    let phys_path = golden_data_path("oscillator_test.phys");
    let expected_path = expected_path("oscillator_test.expected");
    
    let result = run_phys_file(phys_path.to_str().unwrap())
        .expect("Failed to run oscillator_test.phys");
    
    let actual_json = result_to_json(&result);
    
    match load_expected(expected_path.to_str().unwrap()) {
        Ok(expected_json) => {
            assert_eq!(actual_json, expected_json,
                "Oscillator output does not match expected snapshot");
        }
        Err(_) => {
            eprintln!("Writing expected file for first time: {:?}", expected_path);
            write_expected(expected_path.to_str().unwrap(), &actual_json)
                .expect("Failed to write expected file");
        }
    }
}

#[test]
fn test_spring_equilibrium_golden() {
    let phys_path = golden_data_path("spring_equilibrium.phys");
    let expected_path = expected_path("spring_equilibrium.expected");
    
    let result = run_phys_file(phys_path.to_str().unwrap())
        .expect("Failed to run spring_equilibrium.phys");
    
    let actual_json = result_to_json(&result);
    
    match load_expected(expected_path.to_str().unwrap()) {
        Ok(expected_json) => {
            assert_eq!(actual_json, expected_json,
                "Spring equilibrium output does not match expected snapshot");
        }
        Err(_) => {
            eprintln!("Writing expected file for first time: {:?}", expected_path);
            write_expected(expected_path.to_str().unwrap(), &actual_json)
                .expect("Failed to write expected file");
        }
    }
}

// Helper test to regenerate all golden files (for manual use)
#[test]
#[ignore] // Ignored by default, run with --ignored flag
fn regenerate_golden_files() {
    let golden_tests = [
        "simple_particle.phys",
        "oscillator_test.phys",
        "spring_equilibrium.phys",
    ];
    
    for filename in &golden_tests {
        let phys_path = golden_data_path(filename);
        let expected_name = filename.replace(".phys", ".expected");
        let expected_path = expected_path(&expected_name);
        
        if !phys_path.exists() {
            eprintln!("Skipping {} (file not found)", filename);
            continue;
        }
        
        let result = run_phys_file(phys_path.to_str().unwrap())
            .expect(&format!("Failed to run {}", filename));
        
        let json = result_to_json(&result);
        write_expected(expected_path.to_str().unwrap(), &json)
            .expect(&format!("Failed to write expected for {}", filename));
        
        eprintln!("Regenerated: {}", expected_name);
    }
}

