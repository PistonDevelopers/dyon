fn main() {
    for i [2, 4) {
        println(i)
    }
    println(min i [2, 4) { i })
    println(max i [2, 4) { i })
    println(sum i [2, 4) { i })
    println(sift i [2, 8) { clone(i) })
}
