fn render(settings) {
    clear(settings.background_color)
    size := 50
    offset := 1
    for i := 0; i < 10; i += 1 {
        draw(color: [.2, .2, 0, 1], rect: [i * (size + offset), 0, size, size])
    }
}
