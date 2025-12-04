use crate::engine::World;

/// Step the simulation forward by dt using semi-implicit Euler integration
pub fn step(world: &mut World, dt: f32) {
    // First, compute all accelerations
    let accelerations: Vec<_> = (0..world.particles.len())
        .map(|i| world.compute_acceleration(i))
        .collect();

    // Update velocities and positions (semi-implicit Euler: v += a*dt, then x += v*dt)
    for (i, particle) in world.particles.iter_mut().enumerate() {
        particle.vel += accelerations[i] * dt;
        particle.pos += particle.vel * dt;
    }
}
