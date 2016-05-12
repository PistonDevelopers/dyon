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

fn main() {
    list := [309, 203, 39, 18, 80, 3, 12, 8.7,
             309, 205, 39, 13, .3, 3, 12, 2.2,
             2.1, 204, 13, 87, 90, 3, 12, 3.5,
             309, 201, 39, 11, 12, 3, 12, 3.3,
             902, 203, 39, 87, 14, 3, 12, 6.3,
             309, 200, 39, 17, 90, 3, 12, 9.9,
             56, 203, 39, 9900, 1, 3, 12, 4.5,
             309, 900, 39, 87, 90, 3, 12, 809]
    for i 1000 {
        min := min(list)
    }
}
