fn main() {
    background_color := [1; 4]

    set(title: "Square!")
    loop {
        if !next_event() { break }
        if render() {
            clear(background_color)
            draw(color: [1, 0, 0, 1], rect: [0, 0, 50, 50])
        }
    }
}
