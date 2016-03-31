fn main() {
    settings := {
        background_color: [1; 4],
        reload_key: 1073741882, // F1
    }
    loader := new_loader(interval: 0.25)
    source := "source/piston_window/square.rs"
    m := unwrap(load(source))
    set(title: call_ret(m, "title", []))
    loop {
        if !next_event() { break }
        if render() {
            call(m, "render", [settings])
        }
        event(loader: loader, source: source, settings: settings, module: m)
    }
}

fn new_loader_interval(interval) -> {
    return {
        time: 0,
        last_reload: 0,
        reload_interval: clone(interval),
        got_error: false
    }
}

fn should_reload(loader) -> {
    return !loader.got_error
        && ((loader.last_reload + loader.reload_interval) < loader.time)
}

fn event_loader_source_settings_module(loader, source, settings, m) {
    if update() {
        dt := unwrap(update_dt())
        loader.time += dt
        if should_reload(loader) {
            loader.last_reload = loader.time
            new_m := load(source)
            if is_err(new_m) {
                loader.got_error = true
                println(unwrap_err(new_m))
                println(" ~~~ Hit F1 to reload ~~~ ")
            } else {
                loader.got_error = false
                m = unwrap(new_m)
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
