use raylib::prelude::*;

use crate::solvers::{create_solver, Problem};

pub fn gui_main(problem_path: &std::path::Path, solver_name: &str) {
    dbg!(problem_path);
    let problem = Problem::load(problem_path).expect("Failed to read the problem file");
    let data = &problem.data;

    let mut solver = create_solver(solver_name);
    solver.initialize(&problem);

    const WIDTH: i32 = 800;
    const HEIGHT: i32 = 800;
    const MARGIN: i32 = 20;

    let (mut rl, thread) = raylib::init()
        .size(WIDTH + MARGIN * 2, HEIGHT + MARGIN * 2)
        .title(&format!("ICFPC2023 - Dare Ludum - {:#?}", problem_path))
        .build();

    let max_x = data
        .attendees
        .iter()
        .max_by_key(|a| a.x as i32)
        .unwrap()
        .x
        .min(data.room_width)
        .max(data.stage_bottom_left.0 + data.stage_width);
    let max_y = data
        .attendees
        .iter()
        .max_by_key(|a| a.y as i32)
        .unwrap()
        .y
        .min(data.room_height)
        .max(data.stage_bottom_left.1 + data.stage_height);

    let ratio_x = if max_x < WIDTH as f32 {
        1.0
    } else {
        WIDTH as f32 / max_x
    };
    let ratio_y = if max_y < HEIGHT as f32 {
        1.0
    } else {
        HEIGHT as f32 / max_y
    };
    let ratio = ratio_x.min(ratio_y);
    dbg!(ratio);

    let mut solution = None;
    let mut done = false;

    while !rl.window_should_close() {
        // ===== HIT TEST =====
        // TODO

        // ===== INTERACTION =====
        match rl.get_key_pressed() {
            Some(k) => match k {
                KeyboardKey::KEY_SPACE => loop {
                    if !done {
                        let (s, d) = solver.solve_step();
                        solution = Some(s);
                        done = d;
                    }
                    if !rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) || done {
                        break;
                    }
                },
                _ => {}
            },
            None => {}
        }

        // ===== DRAWING =====
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::GRAY);

        d.draw_rectangle(
            MARGIN,
            MARGIN,
            (data.room_width * ratio) as i32,
            (data.room_height * ratio) as i32,
            Color::LIGHTGRAY,
        );

        d.draw_rectangle(
            MARGIN + (data.stage_bottom_left.0 * ratio) as i32,
            MARGIN + (data.stage_bottom_left.1 * ratio) as i32,
            (data.stage_width * ratio) as i32,
            (data.stage_height * ratio) as i32,
            Color::BEIGE,
        );

        for attendee in data.attendees.iter() {
            d.draw_circle(
                MARGIN + (attendee.x * ratio) as i32,
                MARGIN + (attendee.y * ratio) as i32,
                10.0 * ratio,
                Color::BROWN,
            );
        }

        if let Some(solution) = solution.as_ref() {
            for p in &solution.placements {
                if !p.x.is_nan() {
                    d.draw_circle(
                        MARGIN + (p.x * ratio) as i32,
                        MARGIN + (p.y * ratio) as i32,
                        10.0 * ratio,
                        Color::BLUE,
                    );
                }
            }
        }
    }
}
