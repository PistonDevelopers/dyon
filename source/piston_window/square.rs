fn main() {
    background_color := [1; 4]

    set(title: "Square!")

    time := 0
    last_reload := 0
    reload_interval := 0.25
    loop {
        if !next_event() { break }
        if render() {
            clear(background_color)
            draw(color: [1, 0, 0, 1], rect: [0, 0, 50, 50])
        }
        if update() {
            dt := unwrap(update_dt())
            time += dt
            if (last_reload + reload_interval) < time {
                last_reload = clone(time)
                println(time)
            }
        }
    }
}
