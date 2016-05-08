fn main() {
    list := [81, 402, 5, 12, 42]
    // min := min(list)
    min := min i len(list) { list[i] }
    // max := max(list)
    max := max i len(list) { list[i] }
    println("min: " + to_string(min))
    println("max: " + to_string(max))
}

fn min(list) -> {
    min_arg := none()
    min_val := none()
    for i len(list) {
        less := if min_arg == none() { true }
                else if list[i] < unwrap(min_val) { true }
                else { false }
        if less {
            min_arg = some(i)
            min_val = some(list[i])
        }
    }
    return if min_arg == none() { none() }
           else { some([unwrap(min_arg), unwrap(min_val)]) }
}

fn max(list) -> {
    max_arg := none()
    max_val := none()
    for i len(list) {
        more := if max_arg == none() { true }
                else if list[i] > unwrap(max_val) { true }
                else { false }
        if more {
            max_arg = some(i)
            max_val = some(list[i])
        }
    }
    return if max_arg == none() { none() }
           else { some([unwrap(max_arg), unwrap(max_val)]) }
}
