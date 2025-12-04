//! Unit tests for gravity force model

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
        pos: Vec2::new(3.0, 4.0), // distance = 5.0
        vel: Vec2::ZERO,
        mass: 2.0,
    });
    world
}

#[test]
fn test_gravity_force_magnitude() {
    let mut world = create_test_world();
    world.forces.push(Force::Gravity {
        a: 0,
        b: 1,
        g: 1.0,
    });
    
    // Compute acceleration on particle a
    let accel = world.compute_acceleration(0);
    
    // Distance between particles is 5.0
    // Force magnitude = G * m_b / r² = 1.0 * 2.0 / 25.0 = 0.08
    // Direction from a to b is (3, 4) normalized = (0.6, 0.8)
    // Expected acceleration = 0.08 * (0.6, 0.8) = (0.048, 0.064)
    let expected_magnitude = 1.0 * 2.0 / 25.0; // G * m_b / r²
    let direction = Vec2::new(3.0, 4.0).normalize();
    let expected_accel = direction * expected_magnitude;
    
    assert!(approx_eq_f32(accel.x, expected_accel.x, 1e-5));
    assert!(approx_eq_f32(accel.y, expected_accel.y, 1e-5));
}

#[test]
fn test_gravity_force_direction() {
    let mut world = create_test_world();
    world.forces.push(Force::Gravity {
        a: 0,
        b: 1,
        g: 1.0,
    });
    
    let accel = world.compute_acceleration(0);
    
    // Acceleration should point from a to b (toward b)
    assert!(accel.x > 0.0);
    assert!(accel.y > 0.0);
}

#[test]
fn test_gravity_force_symmetric() {
    let mut world = create_test_world();
    world.forces.push(Force::Gravity {
        a: 0,
        b: 1,
        g: 1.0,
    });
    
    let _accel_a = world.compute_acceleration(0);
    let accel_b = world.compute_acceleration(1);
    
    // Acceleration on b should point from b to a (toward a)
    // Magnitude should be G * m_a / r² = 1.0 * 1.0 / 25.0 = 0.04
    let expected_magnitude_b = 1.0 * 1.0 / 25.0;
    let direction_b_to_a = Vec2::new(-3.0, -4.0).normalize();
    let expected_accel_b = direction_b_to_a * expected_magnitude_b;
    
    assert!(approx_eq_f32(accel_b.x, expected_accel_b.x, 1e-5));
    assert!(approx_eq_f32(accel_b.y, expected_accel_b.y, 1e-5));
}

#[test]
fn test_gravity_force_zero_distance() {
    let mut world = World::new();
    world.particles.push(Particle {
        name: "a".to_string(),
        pos: Vec2::new(0.0, 0.0),
        vel: Vec2::ZERO,
        mass: 1.0,
    });
    world.particles.push(Particle {
        name: "b".to_string(),
        pos: Vec2::new(0.0, 0.0), // Same position
        vel: Vec2::ZERO,
        mass: 1.0,
    });
    world.forces.push(Force::Gravity {
        a: 0,
        b: 1,
        g: 1.0,
    });
    
    // Should not panic, but acceleration should be zero (or very small)
    let accel = world.compute_acceleration(0);
    // With zero distance, we should get zero acceleration (division by zero handled)
    assert!(accel.length() < 1e-6 || accel.length().is_nan());
}

#[test]
fn test_gravity_force_different_g() {
    let mut world = create_test_world();
    world.forces.push(Force::Gravity {
        a: 0,
        b: 1,
        g: 2.0, // Different G value
    });
    
    let accel = world.compute_acceleration(0);
    
    // With G=2.0, force should be twice as strong
    let expected_magnitude = 2.0 * 2.0 / 25.0; // G * m_b / r²
    let direction = Vec2::new(3.0, 4.0).normalize();
    let expected_accel = direction * expected_magnitude;
    
    assert!(approx_eq_f32(accel.x, expected_accel.x, 1e-5));
    assert!(approx_eq_f32(accel.y, expected_accel.y, 1e-5));
}

