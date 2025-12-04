# Syntax Reference

This document defines the complete syntax of PhysLang using Extended Backus-Naur Form (EBNF).

## Grammar

```
Program         ::= Statement* EOF ;

Statement       ::= ParticleDecl
                  | ForceDecl
                  | WellDecl
                  | LoopDecl
                  | SimulateDecl
                  | DetectorDecl ;

ParticleDecl    ::= "particle" Ident "at" "(" Float "," Float ")"
                    "mass" Float ;

ForceDecl       ::= "force" ForceSpec ;

ForceSpec       ::= "gravity" "(" Ident "," Ident ")" "G" "=" Float
                  | "spring"  "(" Ident "," Ident ")" "k" "=" Float
                                   "rest" "=" Float
                  | "push"    "(" Ident ")" "magnitude" Float
                                   "direction" "(" Float "," Float ")" ;

WellDecl        ::= "well" Ident "on" Ident
                    "if" ObservableRel
                    "depth" Float ;

LoopDecl        ::= "loop" LoopKind "{" LoopBodyStmt* "}" ;

LoopKind        ::= LoopForCycles
                  | LoopWhile ;

LoopForCycles   ::= "for" Integer "cycles"
                    "with" "frequency" Float
                          "damping"   Float
                    "on" Ident ;

LoopWhile       ::= "while" ConditionExpr
                    "with" "frequency" Float
                          "damping"   Float
                    "on" Ident ;

LoopBodyStmt    ::= "force" "push" "(" Ident ")"
                    "magnitude" Float
                    "direction" "(" Float "," Float ")" ;

SimulateDecl    ::= "simulate" "dt" "=" Float
                    "steps" "=" Integer ;

DetectorDecl    ::= "detect" Ident "=" DetectorExpr ;

DetectorExpr    ::= "position" "(" Ident ")"        // returns x-coordinate in v0.2
                  | "distance" "(" Ident "," Ident ")" ;

ConditionExpr   ::= ObservableRel ;

ObservableRel   ::= Observable (("<" | ">" | "<=" | ">=") Float) ;

Observable      ::= "position" "(" Ident ")" "." ("x" | "y")
                  | "distance" "(" Ident "," Ident ")" ;

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

### Reserved Words

The following are reserved keywords and cannot be used as identifiers:
- `particle`, `force`, `gravity`, `spring`, `push`
- `simulate`, `detect`, `position`, `distance`
- `loop`, `for`, `while`, `cycles`, `with`, `frequency`, `damping`, `on`
- `well`, `if`, `depth`
- `at`, `mass`, `G`, `k`, `rest`, `magnitude`, `direction`
- `dt`, `steps`

