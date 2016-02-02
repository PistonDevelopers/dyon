fn main() {
    a := 3
    b := [a]    // `b` is declared and assigned an array.
                // The array contains a reference to `a`.
    c := b      // `c` is declared and shallow clones `b`.
                // The array contains a reference to `a`.
    a = 4
    println(b)  // prints `[4]`
    println(c)  // prints `[4]`
    b[0] = 2    // `b[0]` is changed to 2, but `c` is unaffected
    c[0] = 5    // `c[0]` is changed to 4, but `b` is unaffected
    println(b)  // prints `{x: 2}`
    println(c)  // prints `{x: 5}`
}
