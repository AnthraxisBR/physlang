//! Unit tests for potential well behavior

use physlang_core::loops::{apply_wells, ObservableRuntime, WellInstance};
use physlang_core::engine::Particle;
use physlang_core::tests::test_helpers::approx_eq_f32;
use glam::Vec2;

#[test]
fn test_well_below_threshold_no_force() {
    let mut particles = vec![Particle {
        name: "a".to_string(),
        pos: Vec2::new(3.0, 0.0), // Below threshold of 5.0
        vel: Vec2::ZERO,
        mass: 1.0,
    }];
    
    let wells = vec![WellInstance {
        particle_index: 0,
        observable: ObservableRuntime::PositionX(0),
        threshold: 5.0,
        depth: 10.0,
    }];
    
    let initial_vel = particles[0].vel;
    apply_wells(&wells, &mut particles, 0.01);
    
    // Below threshold, no force should be applied
    assert_eq!(particles[0].vel, initial_vel);
}

#[test]
fn test_well_above_threshold_applies_force() {
    let mut particles = vec![Particle {
        name: "a".to_string(),
        pos: Vec2::new(7.0, 0.0), // Above threshold of 5.0
        vel: Vec2::ZERO,
        mass: 1.0,
    }];
    
    let wells = vec![WellInstance {
        particle_index: 0,
        observable: ObservableRuntime::PositionX(0),
        threshold: 5.0,
        depth: 10.0,
    }];
    
    let initial_vel = particles[0].vel;
    let dt = 0.01;
    apply_wells(&wells, &mut particles, dt);
    
    // Above threshold, force should pull toward threshold
    // Displacement = 7.0 - 5.0 = 2.0
    // Force = -depth * displacement = -10.0 * 2.0 = -20.0
    // Acceleration = force / mass = -20.0 / 1.0 = -20.0
    // Velocity change = accel * dt = -20.0 * 0.01 = -0.2
    let expected_vel_change = -10.0 * 2.0 / 1.0 * dt;
    
    assert!(approx_eq_f32(particles[0].vel.x, initial_vel.x + expected_vel_change, 1e-5));
}

#[test]
fn test_well_at_threshold() {
    let mut particles = vec![Particle {
        name: "a".to_string(),
        pos: Vec2::new(5.0, 0.0), // Exactly at threshold
        vel: Vec2::ZERO,
        mass: 1.0,
    }];
    
    let wells = vec![WellInstance {
        particle_index: 0,
        observable: ObservableRuntime::PositionX(0),
        threshold: 5.0,
        depth: 10.0,
    }];
    
    let initial_vel = particles[0].vel;
    apply_wells(&wells, &mut particles, 0.01);
    
    // At threshold, displacement = 0, so no force
    assert_eq!(particles[0].vel, initial_vel);
}

#[test]
fn test_well_force_direction() {
    let mut particles = vec![Particle {
        name: "a".to_string(),
        pos: Vec2::new(10.0, 0.0), // Well above threshold of 5.0
        vel: Vec2::ZERO,
        mass: 1.0,
    }];
    
    let wells = vec![WellInstance {
        particle_index: 0,
        observable: ObservableRuntime::PositionX(0),
        threshold: 5.0,
        depth: 10.0,
    }];
    
    apply_wells(&wells, &mut particles, 0.01);
    
    // Force should pull toward threshold (negative x direction)
    assert!(particles[0].vel.x < 0.0);
}

#[test]
fn test_well_position_y() {
    let mut particles = vec![Particle {
        name: "a".to_string(),
        pos: Vec2::new(0.0, 8.0), // Above threshold of 5.0 in y
        vel: Vec2::ZERO,
        mass: 1.0,
    }];
    
    let wells = vec![WellInstance {
        particle_index: 0,
        observable: ObservableRuntime::PositionY(0),
        threshold: 5.0,
        depth: 10.0,
    }];
    
    let dt = 0.01;
    apply_wells(&wells, &mut particles, dt);
    
    // Force should affect y velocity
    let expected_vel_change = -10.0 * 3.0 / 1.0 * dt; // displacement = 8.0 - 5.0 = 3.0
    assert!(approx_eq_f32(particles[0].vel.y, expected_vel_change, 1e-5));
    assert_eq!(particles[0].vel.x, 0.0);
}

#[test]
fn test_well_different_depth() {
    let mut particles = vec![Particle {
        name: "a".to_string(),
        pos: Vec2::new(7.0, 0.0), // Above threshold of 5.0
        vel: Vec2::ZERO,
        mass: 1.0,
    }];
    
    let wells = vec![WellInstance {
        particle_index: 0,
        observable: ObservableRuntime::PositionX(0),
        threshold: 5.0,
        depth: 20.0, // Different depth
    }];
    
    let dt = 0.01;
    apply_wells(&wells, &mut particles, dt);
    
    // With depth=20.0, force should be twice as strong
    let expected_vel_change = -20.0 * 2.0 / 1.0 * dt; // displacement = 2.0
    assert!(approx_eq_f32(particles[0].vel.x, expected_vel_change, 1e-5));
}

#[test]
fn test_well_different_mass() {
    let mut particles = vec![Particle {
        name: "a".to_string(),
        pos: Vec2::new(7.0, 0.0),
        vel: Vec2::ZERO,
        mass: 2.0, // Different mass
    }];
    
    let wells = vec![WellInstance {
        particle_index: 0,
        observable: ObservableRuntime::PositionX(0),
        threshold: 5.0,
        depth: 10.0,
    }];
    
    let dt = 0.01;
    apply_wells(&wells, &mut particles, dt);
    
    // With mass=2.0, acceleration should be half
    let expected_vel_change = -10.0 * 2.0 / 2.0 * dt; // force / mass
    assert!(approx_eq_f32(particles[0].vel.x, expected_vel_change, 1e-5));
}

#[test]
fn test_well_negative_threshold() {
    let mut particles = vec![Particle {
        name: "a".to_string(),
        pos: Vec2::new(-3.0, 0.0), // Above threshold of -5.0
        vel: Vec2::ZERO,
        mass: 1.0,
    }];
    
    let wells = vec![WellInstance {
        particle_index: 0,
        observable: ObservableRuntime::PositionX(0),
        threshold: -5.0,
        depth: 10.0,
    }];
    
    let dt = 0.01;
    apply_wells(&wells, &mut particles, dt);
    
    // Displacement = -3.0 - (-5.0) = 2.0
    // Force should pull toward -5.0 (negative direction)
    let expected_vel_change = -10.0 * 2.0 / 1.0 * dt;
    assert!(approx_eq_f32(particles[0].vel.x, expected_vel_change, 1e-5));
}

