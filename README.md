# PhysLang

**PhysLang** is a domain-specific language where program execution is a 2D physics simulation.


> **Tagline**: "Execution as physics, AI at peak."

## What is PhysLang?

PhysLang is a programming language where:

- **A program defines a 2D physical system**
- **Variables are particles** (mass points with position, velocity, mass)
- **Operations are forces** between particles (gravity, springs, etc.)
- **Control-flow is realized via**:
  - **Oscillators** (loops / repeated events)
  - **Potential wells** (conditional regions)
- **Execution is time integration** of Newtonian equations
- **Output is read via detectors** applied to the final state

**Implementation**: Interpreted, deterministic, fixed-step ODE integration.

## Quick Start

### Installation

PhysLang requires Rust (1.70+). To build:

```bash
cargo build --release
```

### Run Your First Program

```bash
cargo run --bin physlang -- run examples/simple.phys
```

### Minimal Example

```phys
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0

force gravity(a, b) G = 1.0
force spring(a, b) k = 2.0 rest = 3.0

simulate dt = 0.01 steps = 10000

detect a_pos = position(a)
detect dist_ab = distance(a, b)
```

### Example with Expressions and Functions (v0.6+)

```phys
let base_mass = 2.0
let spring_k = 1.5

fn make_particle(name, x, y, mass) {
    particle name at (x, y) mass mass
}

make_particle("a", 0.0, 0.0, base_mass)
make_particle("b", 5.0, 0.0, base_mass * 0.8)

force spring("a", "b") k = spring_k * 2.0 rest = 3.0

simulate dt = 0.01 steps = 10000
```

## Documentation

Comprehensive documentation is available in the [`docs/`](docs/) directory:

- **[Language Overview](docs/language-overview.md)** - Core concepts and formal definitions
- **[Syntax Reference](docs/syntax.md)** - Complete EBNF grammar
- **[Semantics](docs/semantics.md)** - Static and dynamic semantics
- **[Standard Library](docs/standard-library.md)** - Built-in forces, observables, and constructs
- **[Getting Started](docs/getting-started.md)** - Installation and tutorial
- **[Examples](docs/examples.md)** - Example programs and use cases

## Features

### Current (v0.8)

- ✅ Particle declarations with position and mass
- ✅ Forces: gravity, spring, push (in loops)
- ✅ Fixed-step simulation with semi-implicit Euler integration
- ✅ Detectors: position (x-coordinate), distance
- ✅ For-loops via oscillators (physics-level)
- ✅ While-loops via physical conditions (physics-level)
- ✅ Potential wells as conditionals (physics-level)
- ✅ **Visual Evaluation Loop (VEL)** - Interactive visualization with live editing
- ✅ **v0.6: Expressions & Variables**
  - `let` bindings for variables
  - Expression language with arithmetic operators (+, -, *, /)
  - Unary minus operator
  - Built-in functions: `sin`, `cos`, `sqrt`, `clamp`
  - Expressions in all numeric positions (mass, forces, simulation parameters, etc.)
- ✅ **v0.7: User-Defined Functions**
  - Function definitions with parameters
  - Function calls as statements
  - Local `let` bindings within functions
  - Return statements (scalar values)
  - World-building statements inside functions (particles, forces, loops, wells)
  - Nested function calls
- ✅ **v0.8: Language-Level Control Flow**
  - `if condition { } else { }` - conditional world generation
  - `for var in start..end { }` - parametric loops with iterator variables
  - `match expr { pattern => { } }` - pattern matching on integer values
  - Comparison operators: `==`, `!=`, `<`, `>`, `<=`, `>=`
  - Proper scoping for loop variables and block-local bindings

### Planned

- Multi-dimensional position detectors
- Equilibrium-based termination
- Additional force types (drag, attractor, repulsion)
- Energy detectors
- Web-based visualization

## Examples

### Simple Two-Particle System

```bash
cargo run --bin physlang -- run examples/simple.phys
```

### Force-Directed Graph Layout

```bash
cargo run --bin physlang -- run examples/graph_layout.phys
```

### For-Loop Example

```bash
cargo run --bin physlang -- run examples/loop_for_push.phys
```

### While-Loop with Well

```bash
cargo run --bin physlang -- run examples/loop_while_well.phys
```

### Visual Evaluation Loop (VEL)

Launch an interactive visualization of your PhysLang program:

```bash
cargo run --bin physlang -- visual examples/graph_layout.phys
```

The Visual Evaluation Loop (VEL) provides:

- **Live visualization** of particles and forces in a 2D canvas
- **Interactive controls**: Play/Pause, Reset, Step, and speed adjustment
- **File watching**: Edit and save your `.phys` file to automatically reload the simulation
- **Real-time feedback**: See your program execute step-by-step with visual representation
- **Auto-scaling viewport**: Automatically adjusts to keep all particles visible

When you edit and save the source file, VEL will automatically re-parse, re-analyze, and restart the simulation with your updated code. This creates a REPL-like development experience for PhysLang programs.

#### VEL Examples

Try these interactive visualizations:

**Market Stress Demo**:
```bash
cargo run --bin physlang -- visual examples/vel/market_stress_demo.phys
```
Demonstrates how financial stress propagates through a network of companies.

**Systemic Risk Visualizer**:
```bash
cargo run --bin physlang -- visual examples/vel/systemic_risk_visualizer/systemic_risk.phys
```
A comprehensive simulation of systemic risk in an interconnected banking network. See [Examples Documentation](docs/examples.md) for details.

## Project Structure

```
physlang/
├── Cargo.toml              # Workspace configuration
├── README.md               # This file
├── docs/                   # Documentation
│   ├── language-overview.md
│   ├── syntax.md
│   ├── semantics.md
│   ├── standard-library.md
│   ├── getting-started.md
│   └── examples.md
├── physlang-core/          # Core library crate
│   └── src/
│       ├── ast.rs          # Abstract syntax tree
│       ├── parser.rs        # Parser implementation
│       ├── analyzer.rs      # Static analysis and type checking
│       ├── eval.rs          # Expression evaluation (v0.6+)
│       ├── functions.rs     # Function execution (v0.7+)
│       ├── engine.rs        # Physics engine
│       ├── integrator.rs   # Numerical integrator
│       ├── loops.rs         # Loop and well runtime
│       └── runtime.rs       # Runtime execution
├── physlang-cli/           # Command-line interface
│   └── src/
│       ├── main.rs
│       └── vel_app.rs      # Visual Evaluation Loop (VEL) application
└── examples/               # Example programs
    ├── simple.phys
    ├── graph_layout.phys
    ├── loop_for_push.phys
    └── loop_while_well.phys
```

## Physics Model

PhysLang uses **semi-implicit Euler integration**:

```
v(t + Δt) = v(t) + a(t) * Δt
x(t + Δt) = x(t) + v(t + Δt) * Δt
```

**Gravity force**:
```
F = G * m₁ * m₂ / r²
```

**Spring force**:
```
F = k * (distance - rest_length)
```

See [Language Overview](docs/language-overview.md) for formal definitions.

## Contributing

Contributions are welcome! Please see the project structure and coding guidelines in `.cursor/rules/project.mdc`.

## License

[Add your license here]
