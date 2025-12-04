# PhysLang

**PhysLang** is a domain-specific language where program execution is a 2D physics simulation.

> **Tagline**: "Execution as physics."

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

## Documentation

Comprehensive documentation is available in the [`docs/`](docs/) directory:

- **[Language Overview](docs/language-overview.md)** - Core concepts and formal definitions
- **[Syntax Reference](docs/syntax.md)** - Complete EBNF grammar
- **[Semantics](docs/semantics.md)** - Static and dynamic semantics
- **[Standard Library](docs/standard-library.md)** - Built-in forces, observables, and constructs
- **[Getting Started](docs/getting-started.md)** - Installation and tutorial
- **[Examples](docs/examples.md)** - Example programs and use cases

## Features

### Current (v0.5)

- ✅ Particle declarations with position and mass
- ✅ Forces: gravity, spring, push (in loops)
- ✅ Fixed-step simulation with semi-implicit Euler integration
- ✅ Detectors: position (x-coordinate), distance
- ✅ For-loops via oscillators
- ✅ While-loops via physical conditions
- ✅ Potential wells as conditionals
- ✅ **Visual Evaluation Loop (VEL)** - Interactive visualization with live editing

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

When you edit and save the source file, VEL will automatically re-parse, re-analyze, and restart the simulation with your updated code. This creates a REPL-like development experience for PhysLang programs.

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
