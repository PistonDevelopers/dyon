/*
Wraps a nice interface around draw list commands.
*/

fn clear_dlist_color(mut draw_list, color) {
    push(mut draw_list, ["clear", color])
}

fn draw_dlist_color_radius_line(mut draw_list, color, radius, line) {
    push(mut draw_list, ["draw_color_radius_line", color, radius, line])
}

fn draw_dlist_color_rectangle(mut draw_list, color, rectangle) {
    push(mut draw_list, ["draw_color_rectangle", color, rectangle])
}

fn draw_dlist_color_ellipse(mut draw_list, color, ellipse) {
    push(mut draw_list, ["draw_color_ellipse", color, ellipse])
}

fn draw_dlist_color_ellipse_resolution(mut draw_list, color, ellipse, resolution) {
    push(mut draw_list, ["draw_color_ellipse_resolution", color, ellipse, resolution])
}
