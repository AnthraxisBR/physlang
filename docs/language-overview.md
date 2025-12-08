# Language Overview

## Core Concepts

PhysLang is a domain-specific language where program execution is a 2D physics simulation. Instead of executing discrete symbolic instructions, PhysLang defines programs as dynamical systems composed of particles, forces, and detectors.

### Execution Model

- **Program** = A 2D physical system
- **Variables** = Particles (mass points) + Scalar variables (v0.6+)
- **Operations** = Forces between particles
- **Abstraction** = User-defined functions (v0.7+)
- **Control-flow** = Two levels:
  - **Language-level (v0.8+)**: `if/else`, `for`, `match` - evaluated before simulation
  - **Physics-level**: Oscillators (loops) and potential wells (conditionals) - evaluated during simulation
- **Execution** = Time integration of Newtonian equations
- **Output** = Detectors applied to final state

### Implementation Characteristics

- **Interpreted**: Programs are parsed and executed directly
- **Deterministic**: Fixed integrator and time step ensure reproducibility
- **Fixed-step ODE integration**: Uses semi-implicit Euler method
- **Compile-time evaluation** (v0.6+): Expressions and variables are evaluated before simulation
- **Macro-like functions** (v0.7+): Functions generate world-building statements before simulation
- **Two-phase execution** (v0.8+): Language-level control flow (`if`, `for`, `match`) executes before simulation to generate the physical world; physics-level control flow (oscillators, wells) executes during simulation
- **Static type checking** (v0.9+): Types (Scalar, Vec2, Bool, ParticleRef) are checked at compile time
- **Effect system** (v0.9+): Functions are classified as pure or world-building
- **Dimensional analysis** (v0.9+): Optional static checking of physical units consistency

## Formal Definitions

### Particle State

A particle $p$ has state $(x, v, m)$:

- $x \in \mathbb{R}^2$ - position vector
- $v \in \mathbb{R}^2$ - velocity vector  
- $m \in \mathbb{R}_{>0}$ - mass

### World State

A world state $W(t) = \{p_i(t)\}_{i=1..N}$ is a collection of $N$ particles at time $t$.

### Forces

Forces $F_k(W(t))$ are defined over particles and depend on the current world state.

### Time Evolution

The system approximates Newton's second law:

$$m_i \frac{d^2 x_i}{dt^2} = \sum_k F_k^i(W(t))$$

Where $F_k^i$ is the contribution of force $k$ to particle $i$.

### Integration Scheme

PhysLang uses **semi-implicit Euler** integration with fixed time step $\Delta t$:

1. **Compute accelerations**:
   $$a_i(t) = \frac{1}{m_i} \sum_k F_k^i(W(t))$$

2. **Update velocities**:
   $$v_i(t + \Delta t) = v_i(t) + a_i(t) \cdot \Delta t$$

3. **Update positions**:
   $$x_i(t + \Delta t) = x_i(t) + v_i(t + \Delta t) \cdot \Delta t$$

This method provides better energy conservation than explicit Euler while remaining computationally efficient.

### Oscillators (Loops)

Oscillators maintain an internal phase $\phi$ that evolves over time:

$$\phi(t + \Delta t) = \phi(t) + 2\pi f \cdot \Delta t$$

Where $f$ is the frequency. When $\phi \geq 2\pi$, a **loop iteration event** fires:

- Phase resets: $\phi \leftarrow \phi - 2\pi$
- Loop body is executed (e.g., applying impulses)
- For for-loops: cycle counter decrements
- For while-loops: condition is evaluated

**Damping** can be applied to gradually reduce oscillation amplitude:

$$\phi(t + \Delta t) = (\phi(t) + 2\pi f \cdot \Delta t) \cdot (1 - \gamma \cdot \Delta t)$$

Where $\gamma$ is the damping coefficient.

### Potential Wells (Conditionals)

Potential wells apply additional forces when an observable passes a threshold.

For a well with threshold $T$ and depth $D$ on observable $x$:

- If $x \geq T$: Apply restoring force $F = -D \cdot (x - T)$
- This creates a "valley" that attracts the particle toward the threshold

Wells provide a physical mechanism for conditional behavior: particles in the well region (true branch) are pulled toward the threshold, while particles outside (false branch) are not affected.

### Language-Level Control Flow (v0.8+)

PhysLang v0.8 introduces language-level control flow that executes **before simulation**, complementing the physics-level control flow (oscillators and wells) that executes **during simulation**.

#### If/Else Statements

```phys
if condition {
    # true branch
} else {
    # false branch
}
```

Conditions use comparison operators (`==`, `!=`, `<`, `>`, `<=`, `>=`) and evaluate to boolean-like values (0.0 = false, non-zero = true).

#### For Loops

```phys
for i in 0..n {
    # body with i available
}
```

Iterates from start (inclusive) to end (exclusive). The loop variable is available in the body scope.

#### Match Statements

```phys
match expr {
    0 => { # case 0 }
    1 => { # case 1 }
    _ => { # default }
}
```

Pattern matching on integer values. The wildcard `_` matches any value.

These constructs allow **parametric world generation** - creating different physical systems based on configuration parameters, all resolved before the physics simulation begins.

### Detectors

A detector $D$ maps a final world state $W(T)$ to a scalar or vector result:

- `position(a)` → Position vector (or x-coordinate in v0.2)
- `distance(a, b)` → Euclidean distance between particles

Detectors are evaluated once at the end of simulation to extract program outputs.

## Physical Interpretation

### Forces as Operations

- **Gravity**: Attractive force encoding multiplicative relationships
- **Spring**: Restoring force encoding distance constraints
- **Push**: Impulsive force for discrete actions (in loops)

### Oscillators as Iteration

Each oscillator cycle corresponds to one loop iteration. The phase provides a continuous representation of iteration progress, making loops physically grounded rather than discrete jumps.

### Wells as Branching

Potential wells create regions in state space. A particle's position relative to the well determines which "branch" it follows, providing continuous conditional behavior.

### Equilibrium as Termination

Programs terminate after a fixed number of steps. In future versions, termination may occur when the system reaches equilibrium (kinetic energy below threshold).

## Determinism

PhysLang execution is **deterministic** given:

- Fixed integrator algorithm (semi-implicit Euler)
- Fixed time step $\Delta t$
- Fixed initial conditions
- Fixed number of steps

This ensures reproducible results, which is crucial for program correctness and debugging.

## Numerical Considerations

### Time Step Selection

The time step $\Delta t$ must be small enough to maintain stability:

- For oscillators: $\Delta t < \frac{2}{f_{max}}$ where $f_{max}$ is the highest frequency
- For springs: $\Delta t < \frac{2}{\sqrt{k/m}}$ for spring constant $k$ and mass $m$
- Typical values: $\Delta t = 0.01$ to $0.001$

### Energy Conservation

Semi-implicit Euler provides better energy conservation than explicit Euler, but some energy drift is inevitable over long simulations. Damping can help stabilize systems but changes the physics.

### Precision Limits

Floating-point arithmetic limits precision. Very small forces or positions may be lost to numerical precision. Very large values may cause overflow.

## Relation to Analog Computing

PhysLang shares philosophy with analog computers:

- **Continuous evolution**: State evolves smoothly rather than in discrete jumps
- **Physical interpretation**: Computation is directly observable as particle motion
- **Energy-based optimization**: Many problems naturally express as energy minimization

However, PhysLang differs:

- Runs on digital hardware (simulated physics)
- Provides high-level language interface
- Explicitly maps programming constructs to physical phenomena

## Future Directions

- **Higher dimensions**: Extend to 3D or N-dimensional spaces
- **Advanced forces**: Drag, attractors, repulsion, custom potentials
- **Equilibrium detection**: Automatic termination when system stabilizes
- **Visualization**: Real-time rendering of particle motion
- **GPU acceleration**: Parallel force calculations for large systems
- **Differentiable physics**: Integration with machine learning
- **Extended dimensional analysis**: Full algebraic unit inference and user-defined dimensions
- **Dynamic particles**: Runtime particle creation/destruction (with extended lifetime tracking)

