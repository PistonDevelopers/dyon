fn main() {
    settings := {
        background_color: [1; 4]
    }
    source := "source/piston_window/square.rs"
    m := load(source)

    set(title: "Square!")

    time := 0
    last_reload := 0
    reload_interval := 0.25
    loop {
        if !next_event() { break }
        if render() {
            call(m, "render", [settings])
        }
        if update() {
            dt := unwrap(update_dt())
            time += dt
            if (last_reload + reload_interval) < time {
                last_reload = clone(time)
                m = load(source)
            }
        }
    }
}
