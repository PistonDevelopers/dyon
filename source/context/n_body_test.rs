fn main() {
    n := 1000
    bodies := bodies()
    diff := [vec3_zero(); n_pairs()]
    mag := [0; n_pairs()]

    offset_momentum(bodies)
    println(energy(bodies))

    for i := 0; i < n; i += 1 {
        advance(bodies: bodies, dt: 0.01, diff: diff, mag: mag)
    }

    println(energy(bodies))
}
