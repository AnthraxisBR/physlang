//! Unit tests for push force (via loop body)

use physlang_core::engine::Particle;
use physlang_core::loops::LoopBodyRuntime;
use physlang_core::tests::test_helpers::approx_eq_f32;
use glam::Vec2;

#[test]
fn test_push_force_applies_velocity_change() {
    let mut particles = vec![
        Particle {
            name: "a".to_string(),
            pos: Vec2::new(0.0, 0.0),
            vel: Vec2::ZERO,
            mass: 1.0,
        }
    ];
    
    let body = vec![LoopBodyRuntime::ForcePush {
        particle_index: 0,
        magnitude: 2.0,
        direction: Vec2::new(1.0, 0.0),
    }];
    
    // Apply push
    for action in &body {
        match action {
            LoopBodyRuntime::ForcePush {
                particle_index,
                magnitude,
                direction,
            } => {
                let particle = &mut particles[*particle_index];
                let dir_normalized = direction.normalize_or_zero();
                particle.vel += dir_normalized * (*magnitude);
            }
        }
    }
    
    // Velocity should be (2.0, 0.0)
    assert!(approx_eq_f32(particles[0].vel.x, 2.0, 1e-5));
    assert!(approx_eq_f32(particles[0].vel.y, 0.0, 1e-5));
}

#[test]
fn test_push_force_diagonal() {
    let mut particles = vec![
        Particle {
            name: "a".to_string(),
            pos: Vec2::new(0.0, 0.0),
            vel: Vec2::ZERO,
            mass: 1.0,
        }
    ];
    
    let body = vec![LoopBodyRuntime::ForcePush {
        particle_index: 0,
        magnitude: 5.0,
        direction: Vec2::new(3.0, 4.0), // Will be normalized
    }];
    
    // Apply push
    for action in &body {
        match action {
            LoopBodyRuntime::ForcePush {
                particle_index,
                magnitude,
                direction,
            } => {
                let particle = &mut particles[*particle_index];
                let dir_normalized = direction.normalize_or_zero();
                particle.vel += dir_normalized * (*magnitude);
            }
        }
    }
    
    // Direction (3, 4) normalized is (0.6, 0.8)
    // Velocity should be 5.0 * (0.6, 0.8) = (3.0, 4.0)
    assert!(approx_eq_f32(particles[0].vel.x, 3.0, 1e-5));
    assert!(approx_eq_f32(particles[0].vel.y, 4.0, 1e-5));
}

#[test]
fn test_push_force_accumulates() {
    let mut particles = vec![
        Particle {
            name: "a".to_string(),
            pos: Vec2::new(0.0, 0.0),
            vel: Vec2::new(1.0, 0.0), // Initial velocity
            mass: 1.0,
        }
    ];
    
    let body = vec![LoopBodyRuntime::ForcePush {
        particle_index: 0,
        magnitude: 2.0,
        direction: Vec2::new(1.0, 0.0),
    }];
    
    // Apply push
    for action in &body {
        match action {
            LoopBodyRuntime::ForcePush {
                particle_index,
                magnitude,
                direction,
            } => {
                let particle = &mut particles[*particle_index];
                let dir_normalized = direction.normalize_or_zero();
                particle.vel += dir_normalized * (*magnitude);
            }
        }
    }
    
    // Velocity should be 1.0 + 2.0 = 3.0
    assert!(approx_eq_f32(particles[0].vel.x, 3.0, 1e-5));
}

#[test]
fn test_push_force_zero_direction() {
    let mut particles = vec![
        Particle {
            name: "a".to_string(),
            pos: Vec2::new(0.0, 0.0),
            vel: Vec2::ZERO,
            mass: 1.0,
        }
    ];
    
    let body = vec![LoopBodyRuntime::ForcePush {
        particle_index: 0,
        magnitude: 1.0,
        direction: Vec2::ZERO, // Zero direction
    }];
    
    // Apply push
    for action in &body {
        match action {
            LoopBodyRuntime::ForcePush {
                particle_index,
                magnitude,
                direction,
            } => {
                let particle = &mut particles[*particle_index];
                let dir_normalized = direction.normalize_or_zero();
                particle.vel += dir_normalized * (*magnitude);
            }
        }
    }
    
    // With zero direction, velocity should remain zero
    assert!(approx_eq_f32(particles[0].vel.length(), 0.0, 1e-5));
}

