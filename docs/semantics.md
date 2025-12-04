# Semantics

This document defines the static and dynamic semantics of PhysLang.

## Static Semantics

### Type System

PhysLang uses a minimal type system:

**Types**:
- `Scalar` (f32) - Floating-point numbers
- `Vec2` (2D vector) - Position and velocity vectors
- `ParticleRef` - Identifier bound to a particle declaration
- `Bool` - Boolean values for condition evaluation

**Note**: For simplicity, most types are treated as `Scalar` in the implementation, with vector math handled internally.

### Environments

#### Particle Environment

The particle environment $\Gamma_p$ maps identifiers to particle declarations:

$$\Gamma_p : \text{Ident} \rightarrow \text{ParticleDecl}$$

This environment is built during parsing by collecting all `particle` declarations.

### Well-Formedness Rules

A PhysLang program is **well-formed** if it satisfies the following rules:

#### 1. Unique Particle Names

No duplicate particle identifiers:

$$\forall i, j: i \neq j \Rightarrow \text{name}(p_i) \neq \text{name}(p_j)$$

**Error**: "Duplicate particle name: `<name>`"

#### 2. Forces Reference Existing Particles

In `force gravity(a, b) ...` or `force spring(a, b) ...`:

$$a \in \text{dom}(\Gamma_p) \land b \in \text{dom}(\Gamma_p)$$

**Error**: "Particle '`<name>`' not found"

#### 3. Loops Reference Existing Target

In `loop ... on <particle>`:

$$\text{particle} \in \text{dom}(\Gamma_p)$$

**Error**: "Particle '`<name>`' not found for loop"

#### 4. Loop Body Push Target Exists

In `force push(<particle>) ...` inside a loop body:

$$\text{particle} \in \text{dom}(\Gamma_p)$$

**Error**: "Particle '`<name>`' not found"

#### 5. Wells Reference Existing Particle

In `well <name> on <particle> ...`:

$$\text{particle} \in \text{dom}(\Gamma_p)$$

**Error**: "Particle '`<name>`' not found for well"

#### 6. Detectors Reference Existing Particles

In `detect ... = position(<particle>)` or `detect ... = distance(<a>, <b>)`:

$$\text{particle} \in \text{dom}(\Gamma_p) \land a \in \text{dom}(\Gamma_p) \land b \in \text{dom}(\Gamma_p)$$

**Error**: "Particle '`<name>`' not found for detector"

#### 7. Observable References

In conditions like `position(<particle>).x` or `distance(<a>, <b>)`:

$$\text{particle} \in \text{dom}(\Gamma_p) \land a \in \text{dom}(\Gamma_p) \land b \in \text{dom}(\Gamma_p)$$

**Error**: "Particle '`<name>`' not found"

### Observable Typing

Observables have the following types:

- `position(a) : Vec2` (internally, v0.2 returns x-coordinate as Scalar)
- `position(a).x : Scalar`
- `position(a).y : Scalar`
- `distance(a, b) : Scalar`
- `ObservableRel : Bool` (e.g., `position(a).x < 5.0`)

Numeric literals are `Scalar`.

## Dynamic Semantics

### World State

The world state at time $t$ is:

$$W(t) = \{p_i(t)\}_{i=1..N}$$

Where each particle $p_i$ has:
- `pos: Vec2` - Position vector
- `vel: Vec2` - Velocity vector
- `mass: f32` - Mass (positive)

**Implementation**:
```rust
struct World {
    particles: Vec<Particle>,
    forces: Vec<Force>,
}
```

### Force Evaluation

Forces are evaluated at each time step based on the current world state.

#### Gravity Force

For `force gravity(a, b) G = g`:

1. Compute relative position: $r = \text{pos}_b - \text{pos}_a$
2. Compute squared distance: $\text{dist}^2 = \max(|r|^2, \epsilon)$ (avoid division by zero)
3. Compute force magnitude: $f_{\text{mag}} = G \cdot m_a \cdot m_b / \text{dist}^2$
4. Compute force vector: $f_{\text{vec}} = \text{normalize}(r) \cdot f_{\text{mag}}$
5. Apply forces:
   - Add $+f_{\text{vec}}$ to particle $a$
   - Add $-f_{\text{vec}}$ to particle $b$ (Newton's third law)

#### Spring Force

For `force spring(a, b) k = k rest = r`:

1. Compute relative position: $r = \text{pos}_b - \text{pos}_a$
2. Compute distance: $\text{dist} = \max(|r|, \epsilon)$
3. Compute extension: $\text{extension} = \text{dist} - r$ (displacement from rest length)
4. Compute force vector: $f_{\text{vec}} = \text{normalize}(r) \cdot (k \cdot \text{extension})$
5. Apply forces:
   - Add $+f_{\text{vec}}$ to particle $a$
   - Add $-f_{\text{vec}}$ to particle $b$

#### Push Force (in Loop Bodies)

For `force push(a) magnitude m direction (dx, dy)`:

1. Normalize direction: $\hat{d} = \text{normalize}((dx, dy))$
2. Apply impulse: $\text{vel}_a \leftarrow \text{vel}_a + \hat{d} \cdot m$

This is an **instantaneous impulse** applied when the loop iteration fires, not a continuous force.

### Potential Wells

For `well name on a if position(a).x >= T depth D`:

At each time step:

1. Evaluate observable: $x = \text{pos}_a.x$
2. If $x \geq T$:
   - Compute displacement: $\Delta x = x - T$
   - Compute restoring force: $F_x = -D \cdot \Delta x$
   - Apply acceleration: $a_x = F_x / m_a$
   - Update velocity: $\text{vel}_a.x \leftarrow \text{vel}_a.x + a_x \cdot \Delta t$

This creates a spring-like force that pulls the particle toward the threshold when it enters the well region.

**Note**: v0.2 only supports 1D wells along the x-axis. Future versions will support y-axis and distance-based wells.

### Loops (Oscillator-Based)

#### Loop State

Each loop maintains:
- `phase: f32` - Current oscillator phase $\phi \in [0, 2\pi)$
- `frequency: f32` - Oscillation frequency $f$
- `damping: f32` - Damping coefficient $\gamma$
- `active: bool` - Whether the loop is still active

For for-loops:
- `cycles_remaining: u32` - Number of cycles left

For while-loops:
- `condition: ConditionRuntime` - Condition to evaluate

#### Phase Evolution

At each global time step $\Delta t$:

1. **Advance phase**:
   $$\phi(t + \Delta t) = (\phi(t) + 2\pi f \cdot \Delta t) \cdot (1 - \gamma \cdot \Delta t)$$

   The damping term $(1 - \gamma \cdot \Delta t)$ gradually reduces oscillation amplitude.

2. **Check for phase wrap**:
   If $\phi \geq 2\pi$:
   - $\phi \leftarrow \phi - 2\pi$ (reset phase)
   - **Fire loop iteration event**

#### Loop Iteration Event

When a loop iteration fires:

**For for-loops**:
1. Apply loop body (e.g., push impulses)
2. Decrement `cycles_remaining`
3. If `cycles_remaining == 0`: Set `active = false`

**For while-loops**:
1. Evaluate condition based on current world state
2. If condition is `false`: Set `active = false` (do nothing)
3. If condition is `true`: Apply loop body

#### Condition Evaluation

Conditions are evaluated using the current world state:

- `position(a).x < T`: Check if `particles[a].pos.x < T`
- `position(a).x > T`: Check if `particles[a].pos.x > T`
- `position(a).y < T`: Check if `particles[a].pos.y < T`
- `position(a).y > T`: Check if `particles[a].pos.y > T`
- `distance(a, b) < T`: Check if `|particles[a].pos - particles[b].pos| < T`
- `distance(a, b) > T`: Check if `|particles[a].pos - particles[b].pos| > T`

### Simulation Step

A single simulation step with time step $\Delta t$:

1. **Update loops**: Advance oscillators, fire iterations if phase wraps
2. **Apply wells**: Compute well forces and apply to particles
3. **Compute accelerations**: For each particle, sum all forces:
   $$a_i = \frac{1}{m_i} \sum_k F_k^i$$
4. **Update velocities** (semi-implicit Euler):
   $$v_i(t + \Delta t) = v_i(t) + a_i(t) \cdot \Delta t$$
5. **Update positions**:
   $$x_i(t + \Delta t) = x_i(t) + v_i(t + \Delta t) \cdot \Delta t$$
6. **Evaluate loop conditions**: Check while-loop conditions to deactivate finished loops

### Simulation and Termination

A `simulate dt = Δt steps = N` declaration defines:

- Initial time: $t = 0$
- For $i = 1..N$:
  - Perform one simulation step
  - $t \leftarrow t + \Delta t$

**Termination**: The program terminates after exactly $N$ steps.

**Future**: Equilibrium-based termination may be added, where the program stops when kinetic energy falls below a threshold.

### Detector Evaluation

At the end of simulation (after $N$ steps), detectors are evaluated on the final world state $W(T)$ where $T = N \cdot \Delta t$:

**Position detector**:
- `position(a)` → Returns `particles[a].pos.x` (v0.2)
- Future: May return full `Vec2`

**Distance detector**:
- `distance(a, b)` → Returns `|particles[a].pos - particles[b].pos|`

Detectors return a list of `(name, value)` pairs, where `value` is a `Scalar`.

## Operational Semantics Summary

The execution of a PhysLang program can be summarized as:

1. **Parse** → Build AST and particle environment $\Gamma_p$
2. **Validate** → Check well-formedness rules
3. **Build World** → Create initial world state $W(0)$
4. **Build Loops/Wells** → Create loop and well instances
5. **Simulate** → For $i = 1..N$:
   - Update loops
   - Apply wells
   - Integrate physics
   - Evaluate conditions
6. **Detect** → Evaluate detectors on $W(T)$
7. **Output** → Return detector results

This provides a deterministic, physically-grounded execution model where computation emerges from the evolution of a dynamical system.

