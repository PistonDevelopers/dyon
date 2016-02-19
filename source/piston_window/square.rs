fn main() {
    background_color := [1; 4]

    if render() {
        clear(background_color)
        rectangle(color: [1, 0, 0, 1], rect: [0, 0, 100, 100])
    }
}
