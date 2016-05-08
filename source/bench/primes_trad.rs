fn main() {
    a := primes_trad(10000)
}

fn primes_trad(n) -> {
    x := []
    'prime: for i n-2 {
        p := i + 2
        for j sqrt(p)-2 {
            o := j + 2
            if (p % o) == 0 { continue 'prime }
        }
        push(mut x, p)
    }
    return clone(x)
}
