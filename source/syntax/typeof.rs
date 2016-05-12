fn typeof_return() -> {
    if typeof(return) == "return" {
        println("return OK")
    }
    return = 2
    if typeof(return) == "number" {
        println("return is number OK")
    }
}

fn main() {
    if typeof("hi") == "string" {
        println("string OK")
    }
    if typeof(3) == "number" {
        println("number OK")
    }
    if typeof([]) == "array" {
        println("array OK")
    }
    if typeof({}) == "object" {
        println("object OK")
    }
    if typeof(true) == "boolean" {
        println("boolean OK")
    }
    println(typeof_return())
    return
}
