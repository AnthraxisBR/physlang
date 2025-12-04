use glam::Vec2;

/// A PhysLang program AST
#[derive(Debug, Clone)]
pub struct Program {
    pub particles: Vec<ParticleDecl>,
    pub forces: Vec<ForceDecl>,
    pub simulate: SimulateDecl,
    pub detectors: Vec<DetectorDecl>,
    pub loops: Vec<LoopDecl>,      // v0.2
    pub wells: Vec<WellDecl>,       // v0.2
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

// ============================================================================
// v0.2: Loops and Wells
// ============================================================================

/// Loop declaration
#[derive(Debug, Clone)]
pub struct LoopDecl {
    pub name: Option<String>,         // optional loop label, v0.2 can ignore
    pub kind: LoopKind,
    pub body: Vec<LoopBodyStmt>,      // list of actions applied at each iteration
}

/// Loop kinds
#[derive(Debug, Clone)]
pub enum LoopKind {
    ForCycles {
        cycles: u32,
        frequency: f32,
        damping: f32,
        target: String, // particle name
    },
    WhileCondition {
        condition: ConditionExpr,
        frequency: f32,
        damping: f32,
        target: String, // particle name
    },
}

/// Loop body statements (v0.2, minimal)
#[derive(Debug, Clone)]
pub enum LoopBodyStmt {
    ForcePush {
        particle: String,
        magnitude: f32,
        direction: Vec2,
    },
}

/// Condition expressions for while-loops
#[derive(Debug, Clone)]
pub enum ConditionExpr {
    LessThan(ObservableExpr, f32),
    GreaterThan(ObservableExpr, f32),
}

/// Observable expressions (positions, distances)
#[derive(Debug, Clone)]
pub enum ObservableExpr {
    PositionX(String),          // position(a).x
    PositionY(String),          // position(a).y
    Distance(String, String),   // distance(a,b)
}

/// Potential well declaration
#[derive(Debug, Clone)]
pub struct WellDecl {
    pub name: String,
    pub particle: String,
    pub observable: ObservableExpr, // typically PositionX(ident)
    pub threshold: f32,
    pub depth: f32,
}
