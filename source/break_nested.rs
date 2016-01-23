fn main() {
    'a: for i := 0; i < 10; i += 1 {
        'b: for j := 0; j < 10; j += 1 {
            println("hello")
            break 'a
        }
    }
    println("what?")
}
