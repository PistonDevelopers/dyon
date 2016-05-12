fn main() {
    println(" ~~~ SNAKE ~~~ ")
    println("Press A and D to steer.")
    println("Reset with R.")
    println("You can modify \"source/piston_window/snake.rs\" while running.")

    {
        println("External functions:")
        println("===================")
        functions := unwrap(load("examples/common/functions.rs"))
        list := functions()
        call(functions, "print_functions", [list])
        println("===================")
    }

    render := unwrap(load("examples/piston_window/render.rs"))
    source := "examples/piston_window/snake.rs"
    m := unwrap(load(source: source, imports: [render]))

    settings := call_ret(m, "settings", [])
    data := call_ret(m, "init_data", [settings])
    loader := new_loader(interval: settings.reload_interval)
    set(title: call_ret(m, "title", []))
    loop {
        if !next_event() { break }
        if render() {
            call(m, "render", [settings, data])
        }
        if update() {
            dt := unwrap(update_dt())
            // Slow down when window is unfocused.
            dt *= if data.focused { settings.focus_speed } else { settings.unfocus_speed }
            call(m, "update(mut,_,_)", [data, settings, dt])
        }
        event(loader: mut loader, source: source,
              settings: mut settings, module: mut m,
              imports: [render])
        key := press_keyboard_key()
        if key != none() {
            key := unwrap(key)
            if key == settings.reset_key {
                data = call_ret(m, "init_data", [settings])
            } else if key == settings.turn_left {
                data.pressing_left = true
            } else if key == settings.turn_right {
                data.pressing_right = true
            }
        }

        key := release_keyboard_key()
        if key != none() {
            key := unwrap(key)
            if key == settings.turn_left {
                data.pressing_left = false
            } else if key == settings.turn_right {
                data.pressing_right = false
            }
        }

        if focus() {
            data.focused = focus_arg() == some(true)
        }
    }
}

fn new_loader_interval(interval: f64) -> {} {
    return {
        time: 0,
        last_reload: 0,
        reload_interval: clone(interval),
        got_error: false
    }
}

fn should_reload(loader: {}) -> bool {
    return !loader.got_error &&
          ((loader.last_reload + loader.reload_interval) < loader.time)
}

fn event_loader_source_settings_module_imports(
    mut loader: {}, source: str, mut settings: {}, mut m, imports: []
) {
    if update() {
        dt := unwrap(update_dt())
        loader.time += dt
        if should_reload(loader) {
            loader.last_reload = clone(loader.time)
            new_m := load(source: source, imports: imports)
            if is_err(new_m) {
                loader.got_error = true
                println(unwrap_err(new_m))
                println(" ~~~ Hit F1 to reload ~~~ ")
            } else {
                loader.got_error = false
                m = unwrap(new_m)
                settings = call_ret(m, "settings", [])
                loader.reload_interval = clone(settings.reload_interval)
            }
        }
    }
    if press() {
        key := press_keyboard_key()
        if key == some(settings.reload_key) {
            println(" ~~~ Reloading ~~~ ")
            loader.got_error = false
        }
    }
}