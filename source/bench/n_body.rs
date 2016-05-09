fn pi() -> {return 3.141592653589793}
fn solar_mass() -> {return 4 * pi() * pi()}
fn year() -> {return 365.24}
fn n_bodies() -> {return 5}
fn n_pairs() -> {return n_bodies() * (n_bodies() - 1) / 2}

fn vec3(x, y, z) -> {
    return [clone(x), clone(y), clone(z)]
}
fn vec3_zero() -> { return [0, 0, 0] }
fn vec3_norm(self) -> { return sqrt(vec3_squared_norm(self)) }
fn vec3_squared_norm(self) -> {
    return self[0] * self[0] + self[1] * self[1] + self[2] * self[2]
}
fn vec3_add(a, b) -> {
    return [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn vec3_sub(a, b) -> {
    return [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn vec3_mul(a, b) -> {
    return [a[0] * b, a[1] * b, a[2] * b]
}

fn bodies() -> {
    return [
        // Sun
        {
            pos: vec3(0, 0, 0),
            vel: vec3(0, 0, 0),
            mass: solar_mass()
        },
        // Jupiter
        {
            pos: vec3(4.84143144246472090e+00,
                        -1.16032004402742839e+00,
                        -1.03622044471123109e-01),
            vel: vec3(1.66007664274403694e-03 * year(),
                        7.69901118419740425e-03 * year(),
                        -6.90460016972063023e-05 * year()),
            mass: 9.54791938424326609e-04 * solar_mass(),
        },
        // Saturn
        {
            pos: vec3(8.34336671824457987e+00,
                        4.12479856412430479e+00,
                        -4.03523417114321381e-01),
            vel: vec3(-2.76742510726862411e-03 * year(),
                        4.99852801234917238e-03 * year(),
                        2.30417297573763929e-05 * year()),
            mass: 2.85885980666130812e-04 * solar_mass(),
        },
        // Uranus
        {
            pos: vec3(1.28943695621391310e+01,
                        -1.51111514016986312e+01,
                        -2.23307578892655734e-01),
            vel: vec3(2.96460137564761618e-03 * year(),
                        2.37847173959480950e-03 * year(),
                        -2.96589568540237556e-05 * year()),
            mass: 4.36624404335156298e-05 * solar_mass(),
        },
        // Neptune
        {
            pos: vec3(1.53796971148509165e+01,
                        -2.59193146099879641e+01,
                        1.79258772950371181e-01),
            vel: vec3(2.68067772490389322e-03 * year(),
                        1.62824170038242295e-03 * year(),
                        -9.51592254519715870e-05 * year()),
            mass: 5.15138902046611451e-05 * solar_mass(),
        },
    ]
}

/// Computes all pairwise position differences between the planets.
fn pairwise_diffs_bodies_diff(bodies, mut diff) {
    n := len(bodies)
    k := 0
    for i n {
        for j [i+1, n) {
            diff[k] = vec3_sub(bodies[i].pos, bodies[j].pos)
            k += 1
        }
    }
}

/// Computes the magnitude of the force between each pair of planets.
fn magnitudes_diff_dt_mag(diff, dt, mut mag) {
    for i len(diff) {
        d2 := vec3_squared_norm(diff[i])
        mag[i] = dt / (d2 * sqrt(d2))
    }
}

/// Updates the velocities of the planets by computing their gravitational
/// accelerations and performing one step of Euler integration.
fn update_velocities_bodies_dt_diff_mag(mut bodies, dt, mut diff, mut mag) {
    pairwise_diffs(bodies: bodies, diff: mut diff)
    magnitudes(diff: diff, dt: dt, mag: mut mag)

    n := len(bodies)
    k := 0
    for i n {
        for j [i+1, n) {
            diff := diff[k]
            mag := mag[k]
            bodies[i].vel = vec3_sub(bodies[i].vel,
                            vec3_mul(diff, bodies[j].mass * mag))
            bodies[j].vel = vec3_add(bodies[j].vel,
                            vec3_mul(diff, bodies[i].mass * mag))
            k += 1
        }
    }
}


/// Advances the solar system by one timestep by first updating the
/// velocities and then integrating the positions using the updated velocities.
///
/// Note: the `diff` & `mag` arrays are effectively scratch space. They're
/// provided as arguments to avoid re-zeroing them every time `advance` is
/// called.
fn advance_bodies_dt_diff_mag(mut bodies, dt, mut diff, mut mag) {
    update_velocities(bodies: mut bodies, dt: dt, diff: mut diff, mag: mut mag)
    for i len(bodies) {
        bodies[i].pos = vec3_add(bodies[i].pos, vec3_mul(bodies[i].vel, dt))
    }
}

/// Computes the total energy of the solar system.
fn energy(bodies) -> {
    e := 0.0
    n := len(bodies)
    for i n {
        e += vec3_squared_norm(bodies[i].vel) * bodies[i].mass / 2.0
        m := 0
        for j [i+1, n) {
            m += bodies[j].mass /
                 vec3_norm(vec3_sub(bodies[i].pos, bodies[j].pos))
        }
        e -= bodies[i].mass * m
    }
    return clone(e)
}

/// Offsets the sun's velocity to make the overall momentum of the system zero.
fn offset_momentum(mut bodies) {
    p := vec3_zero()
    for i len(bodies) {
        p = vec3_add(p, vec3_mul(bodies[i].vel, bodies[i].mass))
    }
    bodies[0].vel = vec3_mul(p, (-1.0 / bodies[0].mass))
}

fn main() {
    n := 1000
    bodies := bodies()
    diff := [vec3_zero(); n_pairs()]
    mag := [0; n_pairs()]

    offset_momentum(mut bodies)
    // println(energy(bodies))

    for i n {
        advance(bodies: mut bodies, dt: 0.01, diff: mut diff, mag: mut mag)
    }

    // println(energy(bodies))
}
