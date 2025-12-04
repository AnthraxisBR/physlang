use glam::Vec2;

/// A particle in the physics simulation
#[derive(Debug, Clone)]
pub struct Particle {
    pub name: String,
    pub pos: Vec2,
    pub vel: Vec2,
    pub mass: f32,
}

/// A force acting between particles
#[derive(Debug, Clone)]
pub enum Force {
    Gravity {
        a: usize, // particle index
        b: usize, // particle index
        g: f32,   // gravitational constant
    },
    Spring {
        a: usize, // particle index
        b: usize, // particle index
        k: f32,   // spring constant
        rest: f32, // rest length
    },
}

/// The physics world containing particles and forces
#[derive(Debug)]
pub struct World {
    pub particles: Vec<Particle>,
    pub forces: Vec<Force>,
}

impl World {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            forces: Vec::new(),
        }
    }

    /// Compute the acceleration vector for a particle at the given index
    pub fn compute_acceleration(&self, particle_idx: usize) -> Vec2 {
        let particle = &self.particles[particle_idx];
        let mut accel = Vec2::ZERO;

        for force in &self.forces {
            match force {
                Force::Gravity { a, b, g } => {
                    if *a == particle_idx {
                        let other = &self.particles[*b];
                        let r = other.pos - particle.pos;
                        let dist_sq = r.length_squared();
                        if dist_sq > 0.0 {
                            let force_mag = g * other.mass / dist_sq;
                            accel += r.normalize() * force_mag;
                        }
                    } else if *b == particle_idx {
                        let other = &self.particles[*a];
                        let r = other.pos - particle.pos;
                        let dist_sq = r.length_squared();
                        if dist_sq > 0.0 {
                            let force_mag = g * other.mass / dist_sq;
                            accel += r.normalize() * force_mag;
                        }
                    }
                }
                Force::Spring { a, b, k, rest } => {
                    if *a == particle_idx {
                        let other = &self.particles[*b];
                        let r = other.pos - particle.pos;
                        let dist = r.length();
                        if dist > 0.0 {
                            let displacement = dist - rest;
                            let force_mag = k * displacement;
                            accel += r.normalize() * force_mag / particle.mass;
                        }
                    } else if *b == particle_idx {
                        let other = &self.particles[*a];
                        let r = other.pos - particle.pos;
                        let dist = r.length();
                        if dist > 0.0 {
                            let displacement = dist - rest;
                            let force_mag = k * displacement;
                            accel += r.normalize() * force_mag / particle.mass;
                        }
                    }
                }
            }
        }

        accel
    }
}
