use raylib::prelude::*;

use crate::solvers::Problem;

pub fn gui_main(problem_path: &std::path::Path) {
    dbg!(problem_path);
    let problem = Problem::load(problem_path).expect("Failed to read the problem file");
    let data = &problem.data;

    const WIDTH: i32 = 800;
    const HEIGHT: i32 = 800;

    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title(&format!("ICFPC2023 - Dare Ludum - {:#?}", problem_path))
        .build();

    let ratio_x = if data.room_width < WIDTH as f32 {
        1.0
    } else {
        WIDTH as f32 / data.room_width
    };
    let ratio_y = if data.room_height < HEIGHT as f32 {
        1.0
    } else {
        HEIGHT as f32 / data.room_height
    };
    let ratio = ratio_x.min(ratio_y);
    dbg!(ratio);

    while !rl.window_should_close() {
        // ===== HIT TEST =====
        // TODO

        // ===== INTERACTION =====
        // TODO

        // ===== DRAWING =====
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::GRAY);

        d.draw_rectangle(
            0,
            0,
            (data.room_width * ratio) as i32,
            (data.room_height * ratio) as i32,
            Color::LIGHTGRAY,
        );

        d.draw_rectangle(
            (data.stage_bottom_left.0 * ratio) as i32,
            (data.stage_bottom_left.1 * ratio) as i32,
            (data.stage_width * ratio) as i32,
            (data.stage_height * ratio) as i32,
            Color::GRAY,
        );

        for attendee in data.attendees.iter() {
            d.draw_circle(
                (attendee.x * ratio) as i32,
                (attendee.y * ratio) as i32,
                5.0 * ratio,
                Color::BROWN,
            );
        }
    }
}
