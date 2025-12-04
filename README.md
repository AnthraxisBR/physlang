# PhysLang

**PhysLang** is a programming language where program execution is a 2D physics simulation. Instead of executing discrete symbolic instructions, PhysLang defines programs as dynamical systems composed of particles, forces, and detectors.

## Overview

In PhysLang:
- **Variables** are represented as particles (with position, velocity, and mass)
- **Operations** are realized through physical forces (gravity, springs, etc.)
- **Program execution** is the temporal evolution of the physical system
- **Outputs** are extracted through detector functions that measure observables

## Installation

PhysLang requires Rust (1.70+). To build:

```bash
cargo build --release
```

## Usage

Run a PhysLang program:

```bash
cargo run --bin physlang -- run <file.phys>
```

Or after building:

```bash
./target/release/physlang run <file.phys>
```

## Language Syntax (v0.1)

### Particles

Declare particles with initial position and mass:

```
particle name at (x, y) mass m
```

### Forces

**Gravity** between two particles:

```
force gravity(a, b) G = g_value
```

**Spring** connecting two particles:

```
force spring(a, b) k = k_value rest = rest_length
```

### Simulation

Configure the physics integrator:

```
simulate dt = timestep steps = num_steps
```

### Detectors

Extract values from the final world state:

```
detect name = position(particle)
detect name = distance(a, b)
```

## Examples

### Simple Two-Particle System

See `examples/simple.phys` for a basic example with two particles connected by gravity and a spring.

### Force-Directed Graph Layout

The `examples/graph_layout.phys` example demonstrates how PhysLang can compute a force-directed graph layout using only physics primitives.

#### Concept

In this example:
- **Graph nodes** are represented as particles
- **Graph edges** are represented as spring forces
- A **heavy center particle** provides gravity to keep the layout centered
- The simulation runs until forces reach equilibrium
- Final positions approximate an optimal graph layout

#### Graph Structure

The example implements a 5-node graph:
- Nodes: A, B, C, D, E
- Edges: (A-B), (A-C), (B-D), (C-D), (C-E)

#### How It Works

1. Each graph node is initialized as a particle at a starting position
2. Springs connect particles that share an edge, pulling them to a preferred distance
3. Gravity from a heavy center particle prevents the graph from drifting
4. The simulation runs for a fixed number of steps, allowing the system to relax
5. Detectors extract the final x-coordinates and distances

#### Running the Example

```bash
cargo run --bin physlang -- run examples/graph_layout.phys
```

#### Output Interpretation

The output shows:
- `A_x`, `B_x`, `C_x`, `D_x`, `E_x`: Final x-coordinates of each node
- `dist_AB`, `dist_AC`, etc.: Final distances between connected nodes

**Note**: In v0.1, position detectors only return the x-coordinate. In future versions, full 2D position extraction will be available, allowing complete visualization of the graph layout. The current scalar values approximate the 2D layout along the x-axis.

The spring forces will have pulled connected nodes to approximately the rest length (3.0 in this example), while gravity keeps the entire graph centered. The final positions represent a force-directed layout that minimizes spring energy while maintaining compactness.

## Project Structure

```
physlang/
├── Cargo.toml          # Workspace configuration
├── physlang-core/      # Core library crate
│   └── src/
│       ├── ast.rs      # Abstract syntax tree
│       ├── parser.rs   # Parser implementation
│       ├── engine.rs   # Physics engine
│       ├── integrator.rs # Numerical integrator
│       └── runtime.rs  # Runtime execution
└── physlang-cli/       # Command-line interface
    └── src/
        └── main.rs
```

## Physics Details

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

## Version 0.1 Features

- ✅ Particle declarations
- ✅ Gravity and spring forces
- ✅ Fixed-step simulation
- ✅ Position and distance detectors
- ❌ Loops (planned for v0.2)
- ❌ Conditionals (planned for v0.2)
- ❌ Potential wells (planned for v0.2)

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]
