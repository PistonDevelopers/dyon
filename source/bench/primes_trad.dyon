fn main() {
    a := primes_trad(10000)
}

fn primes_trad(n) -> {
    x := []
    'prime: for i [2, n) {
        for j [2, sqrt(i)) { if (i % j) == 0 { continue 'prime } }
        push(mut x, i)
    }
    return clone(x)
}
