fn main() {
    println(primes(100))
}

fn primes(n) -> {
    x := 'prime: sift i n-2 {
        p := i + 2
        for j sqrt(p)-2 {
            o := j + 2
            if (p % o) == 0 { continue 'prime }
        }
        clone(p)
    }
    return clone(x)
}

fn foo() {
    a := sum i 3 { i }
}
