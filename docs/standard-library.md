# Standard Library

PhysLang's "standard library" consists of built-in forces, observables, and control constructs. This document describes each built-in with its physical meaning and usage guidelines.

## Forces

### Gravity

```phys
force gravity(a, b) G = g
```

**Physical Model**: Newtonian gravitational attraction

$$F = G \cdot \frac{m_a \cdot m_b}{r^2}$$

Where:
- $G$ is the gravitational constant
- $m_a, m_b$ are particle masses
- $r$ is the distance between particles

**Parameters**:
- `G` (float): Gravitational constant. Typical range: $0.1$ to $10.0$
  - Smaller values: Weaker attraction, more stable
  - Larger values: Stronger attraction, may require smaller $\Delta t$

**Use Cases**:
- Keeping particles together (centering)
- Encoding multiplicative relationships
- Creating attractive forces

**Stability**: Requires $\Delta t < \frac{2}{\sqrt{G \cdot m_{max}}}$ for stability, where $m_{max}$ is the maximum mass.

**Example**:
```phys
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
force gravity(a, b) G = 1.0
```

### Spring

```phys
force spring(a, b) k = k_value rest = rest_length
```

**Physical Model**: Linear spring (Hooke's law)

$$F = k \cdot (r - r_0)$$

Where:
- $k$ is the spring constant (stiffness)
- $r$ is the current distance
- $r_0$ is the rest length

**Parameters**:
- `k` (float): Spring constant. Typical range: $0.1$ to $100.0$
  - Smaller values: Softer springs, slower response
  - Larger values: Stiffer springs, faster response, may require smaller $\Delta t$
- `rest` (float): Rest length (equilibrium distance). Must be positive.

**Use Cases**:
- Connecting particles at preferred distances
- Graph layout algorithms
- Soft-body simulation
- Constraint satisfaction

**Stability**: Requires $\Delta t < \frac{2}{\sqrt{k/m}}$ for stability, where $m$ is the effective mass.

**Example**:
```phys
particle a at (0.0, 0.0) mass 1.0
particle b at (5.0, 0.0) mass 1.0
force spring(a, b) k = 2.0 rest = 3.0
```

### Push (Loop Bodies Only)

```phys
force push(particle) magnitude m direction (dx, dy)
```

**Physical Model**: Instantaneous impulse

$$\Delta v = \hat{d} \cdot m$$

Where $\hat{d}$ is the normalized direction vector.

**Parameters**:
- `magnitude` (float): Impulse magnitude. Typical range: $0.1$ to $10.0$
- `direction` (Vec2): Direction vector (will be normalized)

**Use Cases**:
- Applying discrete actions in loops
- Pushing particles toward goals
- Creating step-by-step motion

**Note**: Only valid inside loop bodies. Applied as an instantaneous velocity change when the loop iteration fires.

**Example**:
```phys
loop for 10 cycles with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.5 direction (1.0, 0.0)
}
```

## Observables

Observables are expressions that can be evaluated on the current world state.

### Position

```phys
position(particle)
position(particle).x
position(particle).y
```

**Type**: `Vec2` (or `Scalar` for `.x`/`.y`)

**Returns**:
- `position(a)`: Full position vector (v0.2 returns x-coordinate only)
- `position(a).x`: X-coordinate (Scalar)
- `position(a).y`: Y-coordinate (Scalar)

**Use Cases**:
- Detectors: Extract final positions
- Conditions: Check if particle crossed threshold
- Wells: Define well regions

**Example**:
```phys
detect a_x = position(a)
loop while position(a).x < 5.0 with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.3 direction (1.0, 0.0)
}
```

### Distance

```phys
distance(a, b)
```

**Type**: `Scalar`

**Returns**: Euclidean distance between particles $a$ and $b$:

$$d = |\text{pos}_a - \text{pos}_b|$$

**Use Cases**:
- Detectors: Measure final separation
- Conditions: Check if particles are close/far
- Constraints: Verify spring rest lengths

**Example**:
```phys
detect dist_ab = distance(a, b)
loop while distance(a, b) > 10.0 with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.2 direction (1.0, 0.0)
}
```

## Control Constructs

### For-Loop

```phys
loop for N cycles with frequency f damping d on particle {
    <body>
}
```

**Physical Model**: Oscillator-driven iteration

Each loop iteration corresponds to one oscillator cycle. The oscillator maintains a phase $\phi$ that evolves as:

$$\phi(t + \Delta t) = (\phi(t) + 2\pi f \cdot \Delta t) \cdot (1 - d \cdot \Delta t)$$

When $\phi \geq 2\pi$, an iteration fires and $\phi$ resets.

**Parameters**:
- `N` (integer): Number of cycles to execute. Must be positive.
- `frequency` (float): Oscillation frequency in Hz. Typical range: $0.1$ to $10.0$
  - Lower: Slower iterations
  - Higher: Faster iterations, may require smaller $\Delta t$
- `damping` (float): Damping coefficient. Typical range: $0.0$ to $0.1$
  - $0.0$: No damping (constant amplitude)
  - $> 0$: Gradual amplitude reduction
- `particle`: Target particle (for association, not directly used)

**Stability**: Requires $\Delta t < \frac{2}{f}$ for stability.

**Example**:
```phys
loop for 10 cycles with frequency 2.0 damping 0.05 on a {
    force push(a) magnitude 0.3 direction (1.0, 0.0)
}
```

### While-Loop

```phys
loop while <condition> with frequency f damping d on particle {
    <body>
}
```

**Physical Model**: Condition-driven iteration with oscillator

Same oscillator mechanism as for-loops, but iteration only occurs if the condition is true. The condition is evaluated each cycle.

**Parameters**: Same as for-loops, plus:
- `condition`: Boolean expression evaluated each cycle

**Supported Conditions**:
- `position(particle).x < float`
- `position(particle).x > float`
- `position(particle).y < float`
- `position(particle).y > float`
- `distance(a, b) < float`
- `distance(a, b) > float`

**Termination**: Loop deactivates when condition becomes false.

**Example**:
```phys
loop while position(a).x < 5.0 with frequency 1.0 damping 0.0 on a {
    force push(a) magnitude 0.3 direction (1.0, 0.0)
}
```

### Potential Well

```phys
well name on particle if position(particle).x >= threshold depth depth_value
```

**Physical Model**: Region-based attractor

When the observable crosses the threshold, a restoring force is applied:

$$F = -D \cdot (x - T)$$

Where:
- $D$ is the depth (strength)
- $x$ is the observable value
- $T$ is the threshold

**Parameters**:
- `name`: Identifier for the well
- `particle`: Particle the well acts on
- `threshold` (float): Threshold value for the observable
- `depth` (float): Well strength. Typical range: $1.0$ to $100.0$
  - Larger values: Stronger attraction, may require smaller $\Delta t$

**Use Cases**:
- Conditional behavior (particles in well region vs. outside)
- Capturing particles at target positions
- Creating stable regions

**Note**: v0.2 only supports `position(particle).x >= threshold`. Future versions will support more observables and operators.

**Stability**: Requires $\Delta t < \frac{2}{\sqrt{D/m}}$ for stability.

**Example**:
```phys
well target on a if position(a).x >= 5.0 depth 10.0
```

## Simulation Configuration

### Simulate

```phys
simulate dt = timestep steps = num_steps
```

**Parameters**:
- `dt` (float): Time step for integration. Typical range: $0.001$ to $0.1$
  - Smaller: More accurate, slower execution
  - Larger: Faster execution, may be unstable
  - Recommended: $0.01$ for most cases
- `steps` (integer): Number of integration steps. Must be positive.

**Total Simulation Time**: $T = \text{dt} \cdot \text{steps}$

**Example**:
```phys
simulate dt = 0.01 steps = 10000
# Runs for 100 time units
```

## Detectors

### Position Detector

```phys
detect name = position(particle)
```

**Returns**: X-coordinate of particle (v0.2). Future versions may return full `Vec2`.

**Example**:
```phys
detect a_x = position(a)
```

### Distance Detector

```phys
detect name = distance(a, b)
```

**Returns**: Euclidean distance between particles.

**Example**:
```phys
detect dist_ab = distance(a, b)
```

## Parameter Selection Guidelines

### Time Step Selection

Choose $\Delta t$ based on:

1. **Highest frequency**: $\Delta t < \frac{2}{f_{max}}$ where $f_{max}$ is the maximum oscillator frequency
2. **Stiffest spring**: $\Delta t < \frac{2}{\sqrt{k_{max}/m}}$ where $k_{max}$ is the maximum spring constant
3. **Strongest well**: $\Delta t < \frac{2}{\sqrt{D_{max}/m}}$ where $D_{max}$ is the maximum well depth

Use the **most restrictive** constraint.

### Typical Values

- **Time step**: $0.01$ (works for most cases)
- **Gravity constant**: $0.1$ to $1.0$
- **Spring constant**: $1.0$ to $10.0$
- **Well depth**: $5.0$ to $20.0$
- **Oscillator frequency**: $0.5$ to $2.0$
- **Damping**: $0.0$ to $0.1$

## Built-in Functions (v0.6+)

PhysLang provides several built-in mathematical functions that can be used in expressions:

### sin

```phys
sin(expr)
```

**Returns**: Sine of the angle (in radians)

**Example**:
```phys
let angle = 3.14159 / 2.0;
let y = sin(angle);  # y ≈ 1.0
```

### cos

```phys
cos(expr)
```

**Returns**: Cosine of the angle (in radians)

**Example**:
```phys
let angle = 0.0;
let x = cos(angle);  # x = 1.0
```

### sqrt

```phys
sqrt(expr)
```

**Returns**: Square root of the value

**Example**:
```phys
let k = sqrt(2.0);  # k ≈ 1.414
let spring_k = sqrt(mass) * 5.0;
```

### clamp

```phys
clamp(value, min, max)
```

**Returns**: Value clamped between `min` and `max`

**Example**:
```phys
let safe_k = clamp(k, 0.1, 10.0);  # Ensures k is in valid range
let bounded = clamp(position(a).x, -10.0, 10.0);
```

**Use Cases**:
- Ensuring parameters stay in valid ranges
- Bounding values for stability
- Creating safe defaults

## Future Additions

Potential future built-ins:

- **Drag force**: `force drag(particle) coefficient c`
- **Attractor**: `force attractor(particle) center (x, y) strength s`
- **Repulsion**: `force repulsion(a, b) strength s`
- **Energy observables**: `energy_kinetic(a)`, `energy_potential(a)`
- **Speed observable**: `speed(a)`
- **More math functions**: `tan`, `exp`, `log`, `abs`, `min`, `max`

