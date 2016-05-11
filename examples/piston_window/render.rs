/*
Wraps a nice interface around draw list commands.
*/

fn clear_dlist_color(mut draw_list, color) {
    push(mut draw_list, ["clear", color])
}

fn line_dlist_color_radius_from_to(mut draw_list, color, radius, from, to) {
    push(mut draw_list, ["line_color_radius_from_to", color, radius, from, to])
}

fn rectangle_dlist_color_corner_size(mut draw_list, color, corner, size) {
    push(mut draw_list, ["rectangle_color_corner_size", color, corner, size])
}

fn ellipse_dlist_color_corner_size(mut draw_list, color, corner, size) {
    ellipse(dlist: mut draw_list, color: color, corner: corner, size: size, resolution: 16)
}

fn ellipse_dlist_color_corner_size_resolution(mut draw_list, color, corner, size, resolution) {
    push(mut draw_list, ["ellipse_color_corner_size_resolution", color, corner, size, resolution])
}

fn circle_dlist_color_center_radius(mut draw_list, color, center, radius) {
    width := 2 * radius
    ellipse(dlist: mut draw_list, color: color, corner: center - radius, size: (width, width))
}
