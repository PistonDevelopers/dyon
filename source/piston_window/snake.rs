fn title() -> { return "Snake!" }

fn settings() -> {
    return {
        background_color: [1, 1, 0.8, 1],
        reload_interval: 0.25,
        reload_key: 1073741882, // F1
        reset_key: 114, // R
        snake_parts: 100,
        snake_parts_size: 5,
        snake_trail: 10,
        turn_left: 97, // A
        turn_right: 100, // D,
        turn_speed: 5,
        speed: 50,
        focus_speed: 1,
        unfocus_speed: .1,
    }
}

fn init_data(settings) -> {
    data := {
        snake_body: init_snake_body(
            parts: settings.snake_parts,
            size: settings.snake_parts_size
        ),
        snake_angle: 1,
        pressing_left: false,
        pressing_right: false,
        focused: true,
    }
    data.next_snake_body := data.snake_body
    return clone(data)
}

fn init_snake_body_parts_size(parts, size) -> {
    body := []
    // end := [(parts - 1) * size, (parts - 1) * size]
    end := [0, 0]
    for i := 0; i < parts; i += 1 {
        push(mut body, [end[0] - i * size, end[1] - i * size])
    }
    return clone(body)
}

fn render(settings, data) {
    size := 40
    offset := 1
    n := len(data.snake_body)
    d := []
    clear(dlist: mut d, color: settings.background_color)
    for i := 1; i < n; i += 1 {
        pos := data.snake_body[i]
        prev_pos := data.snake_body[i - 1]
        draw(dlist: mut d, color: [.2, .2, 0, 1], radius: 1,
            line: [prev_pos[0], prev_pos[1], pos[0], pos[1]])
    }
    for i := 0; i < n; i += 1 {
        pos := data.snake_body[i]
        draw(dlist: mut d, color: [.2, .2, 0, 1], ellipse: [
            pos[0] - 0.5 * size, pos[1] - 0.5 * size, size, size])
    }
    if n > 0 {
        dir_len := 20
        pos := data.snake_body[0]
        pos2 := [
            pos[0] + cos(data.snake_angle) * dir_len,
            pos[1] + sin(data.snake_angle) * dir_len
        ]
        draw(dlist: mut d, color: [0, 0, 1, 1], radius: 1, line: [pos[0], pos[1], pos2[0], pos2[1]])
    }

    red := [0, 0.4, 0, 1]
    laser := [1, 0, 0, 1]

    walls := [
        [red, [100, 0, 200, 100]],
        [red, [200, 100, 200, 200]],
        [red, [300, 100, 300, 200]],
        [laser, [300, 200, 400, 200]]
    ]

    for i len(walls) {
        draw(dlist: mut d, color: walls[i][0], radius: 5, line: walls[i][1])
    }

    draw(d)
}

fn update(mut data, settings, dt) {
    if data.pressing_left {
        data.snake_angle -= settings.turn_speed * dt
    }
    if data.pressing_right {
        data.snake_angle += settings.turn_speed * dt
    }
    // Update snake body.
    n := len(data.snake_body)
    for i := 0; i < n; i += 1 {
        pos := data.snake_body[i]
        speed := dt * settings.speed
        dir := if i == 0 {
                [cos(data.snake_angle), sin(data.snake_angle)]
            } else {
                prev_pos := data.snake_body[i - 1]
                diff := [prev_pos[0] - pos[0], prev_pos[1] - pos[1]]
                len := sqrt(diff[0]^2 + diff[1]^2)
                if len > settings.snake_parts_size {
                    [diff[0]/len, diff[1]/len]
                } else {
                    [0, 0]
                }
            }
        data.next_snake_body[i] = [
            pos[0] + dir[0] * speed,
            pos[1] + dir[1] * speed
        ]
    }
    data.snake_body = clone(data.next_snake_body)
}
