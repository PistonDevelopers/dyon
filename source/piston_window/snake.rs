fn title() -> { return "Snake!" }

fn settings() -> {
    return {
        background_color: [1; 4],
        reload_interval: 0.25,
        reload_key: 1073741882, // F1
        reset_key: 114, // R
        snake_parts: 100,
        snake_parts_size: 5,
        snake_trail: 10,
        turn_left: 97, // A
        turn_right: 100, // D,
        turn_speed: 3,
        speed: 50,
        focus_speed: 1,
        unfocus_speed: .1,
    }
}

fn render(settings, data) {
    clear(settings.background_color)
    size := 4
    offset := 1
    n := len(data.snake_body)
    for i := 1; i < n; i += 1 {
        pos := data.snake_body[i]
        prev_pos := data.snake_body[i - 1]
        draw(color: [.2, .2, 0, 1], radius: 1,
            line: [prev_pos[0], prev_pos[1], pos[0], pos[1]])
    }
    for i := 0; i < n; i += 1 {
        pos := data.snake_body[i]
        draw(color: [.2, .2, 0, 1], rectangle: [
            pos[0] - 0.5 * size, pos[1] - 0.5 * size, size, size])
    }
    if n > 0 {
        dir_len := 20
        pos := data.snake_body[0]
        pos2 := [
            pos[0] + cos(data.snake_angle) * dir_len,
            pos[1] + sin(data.snake_angle) * dir_len
        ]
        draw(color: [0, 0, 1, 1], radius: 1, line: [pos[0], pos[1], pos2[0], pos2[1]])
    }
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
    data.snake_body = data.next_snake_body
}
