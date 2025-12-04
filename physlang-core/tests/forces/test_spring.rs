//! Unit tests for spring force model

use physlang_core::engine::{Force, Particle, World};
use physlang_core::tests::test_helpers::approx_eq_f32;
use glam::Vec2;

fn create_test_world() -> World {
    let mut world = World::new();
    world.particles.push(Particle {
        name: "a".to_string(),
        pos: Vec2::new(0.0, 0.0),
        vel: Vec2::ZERO,
        mass: 1.0,
    });
    world.particles.push(Particle {
        name: "b".to_string(),
        pos: Vec2::new(5.0, 0.0), // distance = 5.0
        vel: Vec2::ZERO,
        mass: 1.0,
    });
    world
}

#[test]
fn test_spring_force_at_rest_length() {
    let mut world = create_test_world();
    world.forces.push(Force::Spring {
        a: 0,
        b: 1,
        k: 10.0,
        rest: 5.0, // Rest length equals current distance
    });
    
    // At rest length, spring should apply no force
    let accel = world.compute_acceleration(0);
    assert!(accel.length() < 1e-5, "Spring at rest should have zero acceleration");
}

#[test]
fn test_spring_force_extended() {
    let mut world = create_test_world();
    world.forces.push(Force::Spring {
        a: 0,
        b: 1,
        k: 10.0,
        rest: 3.0, // Rest length is 3.0, current distance is 5.0
    });
    
    // Extension = 5.0 - 3.0 = 2.0
    // Force magnitude = k * extension = 10.0 * 2.0 = 20.0
    // Direction from a to b is (1, 0)
    // Acceleration = force / mass = 20.0 / 1.0 = 20.0 in direction (1, 0)
    let accel = world.compute_acceleration(0);
    
    assert!(approx_eq_f32(accel.x, 20.0, 1e-5));
    assert!(approx_eq_f32(accel.y, 0.0, 1e-5));
}

#[test]
fn test_spring_force_compressed() {
    let mut world = create_test_world();
    // Move particles closer
    world.particles[1].pos = Vec2::new(2.0, 0.0); // distance = 2.0
    world.forces.push(Force::Spring {
        a: 0,
        b: 1,
        k: 10.0,
        rest: 5.0, // Rest length is 5.0, current distance is 2.0
    });
    
    // Extension = 2.0 - 5.0 = -3.0 (compressed)
    // Force magnitude = k * extension = 10.0 * (-3.0) = -30.0
    // Direction from a to b is (1, 0)
    // Acceleration = force / mass = -30.0 / 1.0 = -30.0 in direction (1, 0)
    // But wait, for compressed spring, force should push particles apart
    // So acceleration on a should be negative (toward -x)
    let accel = world.compute_acceleration(0);
    
    assert!(accel.x < 0.0, "Compressed spring should push particles apart");
    assert!(approx_eq_f32(accel.y, 0.0, 1e-5));
}

#[test]
fn test_spring_force_direction() {
    let mut world = create_test_world();
    world.forces.push(Force::Spring {
        a: 0,
        b: 1,
        k: 10.0,
        rest: 3.0,
    });
    
    let accel_a = world.compute_acceleration(0);
    let accel_b = world.compute_acceleration(1);
    
    // Acceleration on a should point toward b (positive x)
    assert!(accel_a.x > 0.0);
    // Acceleration on b should point toward a (negative x)
    assert!(accel_b.x < 0.0);
}

#[test]
fn test_spring_force_different_k() {
    let mut world = create_test_world();
    world.forces.push(Force::Spring {
        a: 0,
        b: 1,
        k: 20.0, // Different k value
        rest: 3.0,
    });
    
    // Extension = 5.0 - 3.0 = 2.0
    // Force magnitude = 20.0 * 2.0 = 40.0
    let accel = world.compute_acceleration(0);
    
    assert!(approx_eq_f32(accel.x, 40.0, 1e-5));
}

#[test]
fn test_spring_force_diagonal() {
    let mut world = World::new();
    world.particles.push(Particle {
        name: "a".to_string(),
        pos: Vec2::new(0.0, 0.0),
        vel: Vec2::ZERO,
        mass: 1.0,
    });
    world.particles.push(Particle {
        name: "b".to_string(),
        pos: Vec2::new(3.0, 4.0), // distance = 5.0
        vel: Vec2::ZERO,
        mass: 1.0,
    });
    world.forces.push(Force::Spring {
        a: 0,
        b: 1,
        k: 10.0,
        rest: 3.0, // Rest = 3.0, current = 5.0
    });
    
    // Extension = 5.0 - 3.0 = 2.0
    // Force magnitude = 10.0 * 2.0 = 20.0
    // Direction from a to b is (3, 4) normalized = (0.6, 0.8)
    // Acceleration = 20.0 * (0.6, 0.8) = (12.0, 16.0)
    let accel = world.compute_acceleration(0);
    let direction = Vec2::new(3.0, 4.0).normalize();
    let expected_accel = direction * 20.0;
    
    assert!(approx_eq_f32(accel.x, expected_accel.x, 1e-5));
    assert!(approx_eq_f32(accel.y, expected_accel.y, 1e-5));
}

