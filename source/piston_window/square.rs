fn main() {
    background_color := [1; 4]

    loop {
        if !next_event() { break }
        if render() {
            clear(background_color)
        }
    }
}


/*
fn main() {
    background_color := [1; 4]

    if render() {
        clear(background_color)
        square(100, 100)
    }
}

fn square(x, y) {
    s := 20
    for i := 0; i < 10; i += 1 {
        rectangle(color: [0.7 * random(), 0.5 * random(), 1 * random(), 0.1],
            rect: [i * s + x - 50, i * s + y - 50, 100, 100])
    }
}
*/
