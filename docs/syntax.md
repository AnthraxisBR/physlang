# Syntax Reference

This document defines the complete syntax of PhysLang using Extended Backus-Naur Form (EBNF).

## Grammar

```
Program         ::= (ImportDecl | ModuleDecl | LetDecl | FunctionDecl | TopLevelCall)*
                    (ParticleDecl | ForceDecl | WellDecl | LoopDecl | SimulateDecl | DetectorDecl)*
                    EOF ;

// v0.10: Module system
ModuleDecl      ::= "module" Ident "{" ModuleBody "}" ;

ModuleBody      ::= (ImportDecl | ModuleDecl | LetDecl | FunctionDecl |
                     ParticleDecl | ForceDecl | WellDecl | LoopDecl | DetectorDecl)* ;

ImportDecl      ::= "import" ModulePath ("as" Ident)? ";" ;

ModulePath      ::= Ident ("." Ident)* ;

QualifiedName   ::= Ident ("." Ident)* ;

LetDecl         ::= "let" Ident "=" Expr ";" ;

FunctionDecl    ::= "fn" Ident "(" ParamList? ")" EffectAnnot? "{" Stmt* "}" ;

EffectAnnot     ::= "world" ;  // v0.9: effect annotation for world-building functions

ParamList       ::= Ident ("," Ident)* ;

TopLevelCall    ::= Ident "(" ArgList? ")" ";" ;

ArgList         ::= Expr ("," Expr)* ;

Stmt            ::= LetDecl
                  | ExprCall
                  | ParticleDecl
                  | ForceDecl
                  | WellDecl
                  | LoopDecl
                  | DetectorDecl
                  | ReturnStmt
                  | IfStmt          // v0.8
                  | ForStmt         // v0.8
                  | MatchStmt ;     // v0.8

// v0.8: Language-level control flow
IfStmt          ::= "if" Expr "{" Stmt* "}" ("else" "{" Stmt* "}")? ;

ForStmt         ::= "for" Ident "in" Expr ".." Expr "{" Stmt* "}" ;

MatchStmt       ::= "match" Expr "{" MatchArm* "}" ;

MatchArm        ::= MatchPattern "=>" "{" Stmt* "}" ;

MatchPattern    ::= Integer | "_" ;

ExprCall        ::= Ident "(" ArgList? ")" ";" ;

ReturnStmt      ::= "return" Expr ";" ;

ParticleDecl    ::= "particle" Ident "at" "(" Expr "," Expr ")"
                    "mass" Expr ;

ForceDecl       ::= "force" ForceSpec ;

ForceSpec       ::= "gravity" "(" Ident "," Ident ")" "G" "=" Expr
                  | "spring"  "(" Ident "," Ident ")" "k" "=" Expr
                                   "rest" "=" Expr
                  | "push"    "(" Ident ")" "magnitude" Expr
                                   "direction" "(" Expr "," Expr ")" ;

WellDecl        ::= "well" Ident "on" Ident
                    "if" ObservableRel
                    "depth" Expr ;

LoopDecl        ::= "loop" LoopKind "{" LoopBodyStmt* "}" ;

LoopKind        ::= LoopForCycles
                  | LoopWhile ;

LoopForCycles   ::= "for" Expr "cycles"
                    "with" "frequency" Expr
                          "damping"   Expr
                    "on" Ident ;

LoopWhile       ::= "while" ConditionExpr
                    "with" "frequency" Expr
                          "damping"   Expr
                    "on" Ident ;

LoopBodyStmt    ::= "force" "push" "(" Ident ")"
                    "magnitude" Expr
                    "direction" "(" Expr "," Expr ")" ;

SimulateDecl    ::= "simulate" "dt" "=" Expr
                    "steps" "=" Expr ;

DetectorDecl    ::= "detect" Ident "=" DetectorExpr ;

DetectorExpr    ::= "position" "(" Ident ")"        // returns x-coordinate in v0.2
                  | "distance" "(" Ident "," Ident ")" ;

ConditionExpr   ::= ObservableRel ;

ObservableRel   ::= Observable (("<" | ">" | "<=" | ">=") Expr) ;

Observable      ::= "position" "(" Ident ")" "." ("x" | "y")
                  | "distance" "(" Ident "," Ident ")" ;

Expr            ::= ExprCompare ;    // v0.8: comparison at lowest precedence

ExprCompare     ::= ExprAdd (CompareOp ExprAdd)? ;  // v0.8

CompareOp       ::= "==" | "!=" | "<" | ">" | "<=" | ">=" ;  // v0.8

ExprAdd         ::= ExprMul (("+" | "-") ExprMul)* ;

ExprMul         ::= ExprUnary (("*" | "/") ExprUnary)* ;

ExprUnary       ::= "-" ExprUnary
                  | ExprPrimary ;

ExprPrimary     ::= Float
                  | Integer
                  | String          // v0.8
                  | QualifiedName   // v0.10: supports module.name references
                  | FuncCall
                  | "(" Expr ")" ;

String          ::= '"' [^"]* '"' ;  // v0.8: string literals for particle names

FuncCall        ::= FuncName "(" ArgList? ")" ;

FuncName        ::= BuiltinFunc | QualifiedName ;  // v0.10: user functions can be qualified

BuiltinFunc     ::= "sin" | "cos" | "sqrt" | "clamp" ;

Ident           ::= [A-Za-z_][A-Za-z0-9_]* ;

Float           ::= ("+" | "-")? Digit+ ("." Digit+)? (("e" | "E") ("+" | "-")? Digit+)? ;

Integer         ::= Digit+ ;

Digit           ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;

Whitespace      ::= (" " | "\t" | "\r" | "\n")+ ;

Comment         ::= "#" [^\n]* ;
```

## Lexical Rules

### Identifiers

- Must start with a letter or underscore
- Can contain letters, digits, and underscores
- Case-sensitive
- Examples: `a`, `particle1`, `my_particle`, `_temp`

### Numbers

**Floats**:
- Optional sign (`+` or `-`)
- Integer part (one or more digits)
- Optional fractional part (`.` followed by digits)
- Optional scientific notation (`e` or `E` followed by optional sign and digits)
- Examples: `1.0`, `-3.14`, `2e-3`, `0.01`

**Integers**:
- One or more digits
- Examples: `0`, `42`, `10000`

### Comments

Comments start with `#` and continue to the end of the line:

```phys
# This is a comment
particle a at (0.0, 0.0) mass 1.0  # Inline comment
```

### Whitespace

Whitespace (spaces, tabs, newlines) is used to separate tokens. Multiple whitespace characters are treated as a single separator.

## Statement Types

### Particle Declaration

```phys
particle <name> at (<x>, <y>) mass <m>
```

Declares a particle with:
- `name`: Identifier for the particle
- `x, y`: Initial position coordinates (floats)
- `m`: Mass (positive float)

**Example**:
```phys
particle a at (0.0, 0.0) mass 1.0
particle center at (5.0, 3.0) mass 100.0
```

### Force Declaration

#### Gravity

```phys
force gravity(<a>, <b>) G = <g>
```

Applies gravitational attraction between particles `a` and `b` with gravitational constant `g`.

**Example**:
```phys
force gravity(a, b) G = 1.0
```

#### Spring

```phys
force spring(<a>, <b>) k = <k_value> rest = <rest_length>
```

Connects particles `a` and `b` with a spring:
- `k`: Spring constant (stiffness)
- `rest`: Rest length (equilibrium distance)

**Example**:
```phys
force spring(a, b) k = 2.0 rest = 3.0
```

#### Push (in loop bodies)

```phys
force push(<particle>) magnitude <m> direction (<dx>, <dy>)
```

Applies an impulsive force to a particle. Only valid inside loop bodies.

**Example**:
```phys
loop for 10 cycles with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.5 direction (1.0, 0.0)
}
```

### Well Declaration

```phys
well <name> on <particle> if position(<particle>).x >= <threshold> depth <depth>
```

Creates a potential well that attracts the particle when it crosses the threshold.

**Example**:
```phys
well target on a if position(a).x >= 5.0 depth 10.0
```

**Note**: v0.2 only supports `position(<particle>).x >= <threshold>`. Future versions will support more observables and operators.

### Loop Declaration

#### For-Loop

```phys
loop for <N> cycles with frequency <f> damping <d> on <particle> {
    <loop_body>
}
```

Executes the loop body `N` times, triggered by oscillator cycles.

**Example**:
```phys
loop for 10 cycles with frequency 2.0 damping 0.05 on a {
    force push(a) magnitude 0.3 direction (1.0, 0.0)
}
```

#### While-Loop

```phys
loop while <condition> with frequency <f> damping <d> on <particle> {
    <loop_body>
}
```

Executes the loop body while the condition is true, evaluated each oscillator cycle.

**Example**:
```phys
loop while position(a).x < 5.0 with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.3 direction (1.0, 0.0)
}
```

**Supported conditions**:
- `position(<particle>).x < <float>`
- `position(<particle>).x > <float>`
- `position(<particle>).y < <float>`
- `position(<particle>).y > <float>`
- `distance(<a>, <b>) < <float>`
- `distance(<a>, <b>) > <float>`

### Simulation Declaration

```phys
simulate dt = <timestep> steps = <num_steps>
```

Configures the physics integrator:
- `dt`: Time step for integration (typically 0.01)
- `steps`: Number of integration steps to perform

**Example**:
```phys
simulate dt = 0.01 steps = 10000
```

This runs the simulation for `dt * steps = 0.01 * 10000 = 100` time units.

### Detector Declaration

```phys
detect <name> = position(<particle>)
detect <name> = distance(<a>, <b>)
```

Extracts values from the final world state:
- `position(<particle>)`: Returns x-coordinate (v0.2)
- `distance(<a>, <b>)`: Returns Euclidean distance

**Example**:
```phys
detect a_pos = position(a)
detect dist_ab = distance(a, b)
```

## Syntax Notes

### Order Independence

Statements can appear in any order, except:
- `simulate` must appear before the program can be executed
- Loop bodies must appear between `{` and `}`

### Case Sensitivity

All keywords are **lowercase**: `particle`, `force`, `simulate`, `detect`, `loop`, `well`.

### Multi-line Statements

- Loop bodies can span multiple lines
- Each statement typically appears on one line
- Comments can appear on their own line or after statements

### Expressions (v0.6+)

Expressions can be used in all numeric positions. They support:

**Arithmetic operations**:
- Addition: `+`
- Subtraction: `-`
- Multiplication: `*`
- Division: `/`
- Unary minus: `-expr`

**Operator precedence** (highest to lowest):
1. Unary minus
2. Multiplication, Division
3. Addition, Subtraction

**Built-in functions**:
- `sin(expr)` - Sine (1 argument)
- `cos(expr)` - Cosine (1 argument)
- `sqrt(expr)` - Square root (1 argument)
- `clamp(expr, min, max)` - Clamp value between min and max (3 arguments)

**Examples**:
```phys
let pi = 3.14159;
let half_pi = pi / 2.0;
let k = sqrt(2.0) * 5.0;
let clamped = clamp(x, 0.0, 10.0);
```

### Variables (v0.6+)

**Let bindings**:
```phys
let <name> = <expr>;
```

Declares a variable that can be used in subsequent expressions. Variables are evaluated at "compile-time" (before simulation).

**Example**:
```phys
let g = 9.81;
let mass = 1.0;
particle a at (0.0, 0.0) mass mass
force gravity(a, b) G = g
```

### Functions (v0.7+)

**Function definitions**:
```phys
fn <name>(<param1>, <param2>, ...) {
    <statements>
}
```

Functions can contain:
- Local `let` bindings
- Function calls (including recursive calls)
- World-building statements (particles, forces, loops, wells, detectors)
- `return <expr>;` statements (scalar return values)

**Function calls**:
```phys
<name>(<arg1>, <arg2>, ...);
```

Functions can be called at the top level or inside other functions.

**Example**:
```phys
fn make_particle(name, x, y, m) {
    particle name at (x, y) mass m
}

make_particle(a, 0.0, 0.0, 1.0);
make_particle(b, 5.0, 0.0, 1.0);
```

### Effect Annotations (v0.9+)

Functions can be annotated with `world` to explicitly mark them as world-building:

**Pure functions** (default, no annotation):
```phys
fn compute_distance(x1, y1, x2, y2) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    return sqrt(dx * dx + dy * dy);
}
```

Pure functions:
- Only compute Scalar, Vec2, or Bool values
- Cannot contain particle, force, well, loop, or detector declarations
- Can use `return` to return a value

**World-building functions** (with `world` annotation):
```phys
fn make_chain(n, spacing) world {
    for i in 0..n {
        let x = i * spacing;
        particle p at (x, 0.0) mass 1.0
    }
}
```

World functions:
- May contain world-building statements (particles, forces, wells, loops, detectors)
- Execute before simulation to construct the physical world
- Do not return a value

See [Semantics](semantics.md#effect-system) for detailed effect typing rules.

### Language-Level Control Flow (v0.8+)

v0.8 introduces language-level control flow that operates at "compile time" (during the world-building phase, before simulation), complementing the physics-level control flow (oscillators and wells that execute during simulation).

**Key distinction**:
- **Language-level** (`if`, `for`, `match`): Executes before simulation; generates/skips world declarations
- **Physics-level** (oscillator loops, wells): Executes during simulation; reacts to particle state

See [Semantics: Language-Level Control Flow](semantics.md#language-level-control-flow-v08) for detailed semantics and expansion rules.

#### If Statement

```phys
if <condition> {
    <statements>
} else {
    <statements>
}
```

Conditionally generates world elements based on a **compile-time** condition. The else branch is optional.

**Restrictions**:
- Condition must be a pure expression (no `position()`, `distance()`, or other runtime observables)
- Declarations in inactive branches do not exist in the final world

**Example**:
```phys
let risk_level = 0.7;
if risk_level > 0.5 {
    particle high_risk at (0.0, 0.0) mass 2.0
} else {
    particle low_risk at (0.0, 0.0) mass 1.0
}
```

#### For Loop

```phys
for <var> in <start>..<end> {
    <statements>
}
```

Iterates from `start` (inclusive) to `end` (exclusive), with `var` available in the loop body. The loop is **fully unrolled** at compile time.

**Restrictions**:
- Bounds must be compile-time integer expressions
- Iteration count must be finite and known at compile time
- Particles declared in loops receive mangled names (`p_0`, `p_1`, etc.)

**Example**:
```phys
for i in 0..3 {
    let x = i * 2.0;
    particle p at (x, 0.0) mass 1.0
}
# Creates: p_0 at (0,0), p_1 at (2,0), p_2 at (4,0)
```

#### Match Statement

```phys
match <expr> {
    <pattern> => {
        <statements>
    }
    _ => {
        <statements>
    }
}
```

Matches an expression against integer patterns. The `_` pattern matches anything (wildcard). Only the matching branch is expanded.

**Restrictions**:
- Scrutinee must be a compile-time expression
- Patterns must be integer literals or wildcard (`_`)
- Non-exhaustive matches require a wildcard branch

**Example**:
```phys
let mode = 1;
match mode {
    0 => {
        particle default at (0.0, 0.0) mass 1.0
    }
    1 => {
        particle enhanced at (0.0, 0.0) mass 2.0
    }
    _ => {
        particle fallback at (0.0, 0.0) mass 0.5
    }
}
```

### Comparison Operators (v0.8+)

Expressions support comparison operators for use in conditions:

- `==` - Equal
- `!=` - Not equal
- `<` - Less than
- `>` - Greater than
- `<=` - Less than or equal
- `>=` - Greater than or equal

**Example**:
```phys
if i == 0 {
    # first iteration
}
if x > 5.0 {
    # x is greater than 5
}
```

### String Literals (v0.8+)

String literals are used for dynamic particle names in function calls:

```phys
fn make_particle(name, x, y, m) {
    particle name at (x, y) mass m
}

make_particle("particle_a", 0.0, 0.0, 1.0)
force spring("particle_a", "particle_b") k = 2.0 rest = 3.0
```

### Reserved Words

The following are reserved keywords and cannot be used as identifiers:
- `particle`, `force`, `gravity`, `spring`, `push`
- `simulate`, `detect`, `position`, `distance`
- `loop`, `for`, `while`, `cycles`, `with`, `frequency`, `damping`, `on`
- `well`, `if`, `else`, `depth` (v0.8: `else` added)
- `at`, `mass`, `G`, `k`, `rest`, `magnitude`, `direction`
- `dt`, `steps`
- `let`, `fn`, `return` (v0.6+)
- `sin`, `cos`, `sqrt`, `clamp` (v0.6+)
- `match`, `in` (v0.8+)
- `world` (v0.9+)
- `module`, `import`, `as` (v0.10+)

## Error Handling

PhysLang provides comprehensive error diagnostics for syntax, type, and semantic errors. Common syntax-related errors include:

- **Unexpected token**: Missing semicolons, unbalanced braces, invalid keywords
- **Invalid number format**: Malformed float literals
- **Unterminated string**: Missing closing quote
- **Unknown keyword**: Misspelled reserved words

For complete error documentation including:
- Static type errors
- Effect typing violations
- Dimensional analysis errors
- Validation and runtime errors
- Diagnostic formatting

See [Semantics: Error Semantics and Diagnostics](semantics.md#error-semantics-and-diagnostics).

