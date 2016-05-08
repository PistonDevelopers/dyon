fn main() {
    a := primes(10000)
}

fn primes(n) -> {
    return 'prime: sift i n-2 {
        p := i + 2
        for j sqrt(p)-2 {
            o := j + 2
            if (p % o) == 0 { continue 'prime }
        }
        clone(p)
    }
}
