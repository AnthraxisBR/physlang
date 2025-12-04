// ============================================================================
// v0.6: Expressions & Variables
// ============================================================================

/// Expression AST node
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(f32),
    Var(String),
    UnaryMinus(Box<Expr>),
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        func: FuncName,
        args: Vec<Expr>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Built-in function names
#[derive(Debug, Clone, Copy)]
pub enum FuncName {
    Sin,
    Cos,
    Sqrt,
    Clamp,
}

/// Let binding declaration: `let name = expr`
#[derive(Debug, Clone)]
pub struct LetDecl {
    pub name: String,
    pub expr: Expr,
}

// ============================================================================
// v0.7: User-Defined Functions
// ============================================================================

/// Function declaration: `fn name(params) { body }`
#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

/// Statement AST node
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Local let binding: `let name = expr`
    Let {
        name: String,
        expr: Expr,
    },
    /// Function call statement: `name(args)`
    ExprCall {
        name: String,
        args: Vec<Expr>,
    },
    /// World-building statements
    ParticleDecl(ParticleDecl),
    ForceDecl(ForceDecl),
    LoopDecl(LoopDecl),
    WellDecl(WellDecl),
    DetectorDecl(DetectorDecl),
    /// Return statement: `return expr;`
    Return(Expr),
}

/// A PhysLang program AST
#[derive(Debug, Clone)]
pub struct Program {
    pub lets: Vec<LetDecl>,          // v0.6
    pub functions: Vec<FunctionDecl>, // v0.7
    pub top_level_calls: Vec<Stmt>,  // v0.7: function calls at top level
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
    pub position: (Expr, Expr), // v0.6: x, y as expressions
    pub mass: Expr,             // v0.6: mass as expression
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
    Gravity { g: Expr },           // v0.6: expression
    Spring { k: Expr, rest: Expr }, // v0.6: expressions
}

/// Simulation configuration: `simulate dt = x steps = n`
#[derive(Debug, Clone)]
pub struct SimulateDecl {
    pub dt: Expr,     // v0.6: expression
    pub steps: Expr,  // v0.6: expression (will be coerced to usize)
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
        cycles: Expr,     // v0.6: expression (must evaluate to integer >= 0)
        frequency: Expr,  // v0.6: expression
        damping: Expr,   // v0.6: expression
        target: String,  // particle name
    },
    WhileCondition {
        condition: ConditionExpr,
        frequency: Expr,  // v0.6: expression
        damping: Expr,   // v0.6: expression
        target: String,  // particle name
    },
}

/// Loop body statements (v0.2, minimal)
#[derive(Debug, Clone)]
pub enum LoopBodyStmt {
    ForcePush {
        particle: String,
        magnitude: Expr,              // v0.6: expression
        direction: (Expr, Expr),      // v0.6: x, y as expressions
    },
}

/// Condition expressions for while-loops
#[derive(Debug, Clone)]
pub enum ConditionExpr {
    LessThan(ObservableExpr, Expr),   // v0.6: threshold as expression
    GreaterThan(ObservableExpr, Expr), // v0.6: threshold as expression
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
    pub threshold: Expr,            // v0.6: expression
    pub depth: Expr,                // v0.6: expression
}
