use glam::Vec2;

/// A PhysLang program AST
#[derive(Debug, Clone)]
pub struct Program {
    pub particles: Vec<ParticleDecl>,
    pub forces: Vec<ForceDecl>,
    pub simulate: SimulateDecl,
    pub detectors: Vec<DetectorDecl>,
}

/// Particle declaration: `particle name at (x, y) mass m`
#[derive(Debug, Clone)]
pub struct ParticleDecl {
    pub name: String,
    pub position: Vec2,
    pub mass: f32,
}

/// Force declaration: `force kind(a, b) params...`
#[derive(Debug, Clone)]
pub struct ForceDecl {
    pub a: String,
    pub b: String,
    pub kind: ForceKind,
}

/// Force kinds
#[derive(Debug, Clone)]
pub enum ForceKind {
    Gravity { g: f32 },
    Spring { k: f32, rest: f32 },
}

/// Simulation configuration: `simulate dt = x steps = n`
#[derive(Debug, Clone)]
pub struct SimulateDecl {
    pub dt: f32,
    pub steps: usize,
}

/// Detector declaration: `detect name = kind(...)`
#[derive(Debug, Clone)]
pub struct DetectorDecl {
    pub name: String,
    pub kind: DetectorKind,
}

/// Detector kinds
#[derive(Debug, Clone)]
pub enum DetectorKind {
    Position(String), // particle name
    Distance { a: String, b: String },
}
