fn main() {
    a := 3
    b := {x: a} // `b` is declared and assigned an object.
                // The object contains a reference to `a`.
    c := b      // `c` is declared and shallow clones `b`.
                // The object contains a reference to `a`.
    a = 4
    println(b)  // prints `{x: 4}`
    println(c)  // prints `{x: 4}`
    b.x = 2     // `b.x` is changed to 2, but `c` is unaffected
    c.x = 5     // `c.x` is changed to 4, but `b` is unaffected
    println(b)  // prints `{x: 2}`
    println(c)  // prints `{x: 5}`
}
