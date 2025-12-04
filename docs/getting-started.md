# Getting Started

This guide will help you install PhysLang and run your first program.

## Installation

### Prerequisites

PhysLang requires:
- **Rust** 1.70 or later
- **Cargo** (comes with Rust)

### Install Rust

If you don't have Rust installed:

**Linux/macOS**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows**:
Download and run [rustup-init.exe](https://rustup.rs/)

### Build PhysLang

Clone or navigate to the PhysLang directory:

```bash
cd physlang
cargo build --release
```

This will create the executable at `target/release/physlang`.

## Your First Program

Create a file `hello.phys`:

```phys
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0

force gravity(a, b) G = 1.0
force spring(a, b) k = 2.0 rest = 3.0

simulate dt = 0.01 steps = 10000

detect a_pos = position(a)
detect dist_ab = distance(a, b)
```

Run it:

```bash
cargo run --bin physlang -- run hello.phys
```

Or if you've built it:

```bash
./target/release/physlang run hello.phys
```

You should see output like:

```
a_pos = 1.2345
dist_ab = 3.0001
```

## Understanding the Output

The output shows detector values from the final simulation state:

- `a_pos`: Final x-coordinate of particle `a`
- `dist_ab`: Final distance between particles `a` and `b`

The particles have evolved under the influence of gravity and the spring, reaching an equilibrium state.

## Language Tour

### Particles

Particles are the basic building blocks:

```phys
particle name at (x, y) mass m
```

- `name`: Identifier for the particle
- `(x, y)`: Initial position
- `m`: Mass (must be positive)

**Example**:
```phys
particle ball at (0.0, 0.0) mass 1.0
particle anchor at (5.0, 5.0) mass 100.0
```

### Forces

Forces define interactions between particles:

**Gravity** (attraction):
```phys
force gravity(a, b) G = 1.0
```

**Spring** (connects particles):
```phys
force spring(a, b) k = 2.0 rest = 3.0
```

- `k`: Spring stiffness
- `rest`: Equilibrium distance

### Simulation

Configure how the physics runs:

```phys
simulate dt = 0.01 steps = 10000
```

- `dt`: Time step (smaller = more accurate, slower)
- `steps`: Number of integration steps

Total simulation time = `dt * steps` = `0.01 * 10000` = `100` time units.

### Detectors

Extract values from the final state:

```phys
detect name = position(particle)
detect name = distance(a, b)
```

## Example Programs

### Simple Oscillator

Two particles connected by a spring:

```phys
particle a at (0.0, 0.0) mass 1.0
particle b at (3.0, 0.0) mass 1.0

force spring(a, b) k = 5.0 rest = 3.0

simulate dt = 0.01 steps = 5000

detect dist = distance(a, b)
```

### For-Loop Example

Push a particle multiple times:

```phys
particle a at (0.0, 0.0) mass 1.0

simulate dt = 0.01 steps = 5000

loop for 10 cycles with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.5 direction (1.0, 0.0)
}

detect a_x = position(a)
```

### While-Loop with Well

Push until threshold, then capture:

```phys
particle a at (0.0, 0.0) mass 1.0

well target on a if position(a).x >= 5.0 depth 10.0

simulate dt = 0.01 steps = 8000

loop while position(a).x < 5.0 with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.3 direction (1.0, 0.0)
}

detect a_x = position(a)
```

## Next Steps

1. **Try the examples**: Run programs in the `examples/` directory
2. **Read the documentation**:
   - [Language Overview](language-overview.md) - Core concepts
   - [Syntax Reference](syntax.md) - Complete grammar
   - [Semantics](semantics.md) - How programs execute
   - [Standard Library](standard-library.md) - Built-in functions
3. **Experiment**: Modify examples and see what happens
4. **Build something**: Create your own physics simulation

## Common Issues

### "Particle 'X' not found"

Make sure you've declared the particle before using it:

```phys
# Correct order
particle a at (0.0, 0.0) mass 1.0
force gravity(a, b) G = 1.0  # Error: b not declared yet
particle b at (5.0, 0.0) mass 1.0
```

### Unstable Simulation

If particles fly apart or oscillate wildly:

1. **Reduce time step**: Try `dt = 0.001` instead of `0.01`
2. **Reduce force strengths**: Lower `G`, `k`, or `depth` values
3. **Add damping**: Use `damping = 0.05` in loops

### No Output

Make sure you have:
- A `simulate` declaration
- At least one `detect` declaration
- Valid syntax (check for typos)

## Tips

- **Start simple**: Begin with 2-3 particles
- **Use small time steps**: `dt = 0.01` is usually safe
- **Check stability**: If results are unexpected, try smaller `dt`
- **Read error messages**: They usually point to the problem
- **Experiment**: Physics simulation is interactive - try different values!

## Resources

- **Examples**: See `examples/` directory
- **Documentation**: See other files in `docs/`
- **Paper**: See `PAPER.md` for theoretical background

Happy simulating!

