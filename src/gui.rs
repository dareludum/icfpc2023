use std::path::PathBuf;

use colorgrad::Gradient;
use raylib::prelude::*;

use crate::{
    common::Grid,
    dto::{Attendee, Instrument, SolutionDto},
    scorer::ImpactMap,
    solvers::{create_solver, Problem, Score, Solution},
};

struct ColorGradient {
    neg_gradient: Option<Gradient>,
    pos_gradient: Option<Gradient>,
}

impl ColorGradient {
    pub fn new(min_taste: f64, max_taste: f64) -> Self {
        let mut neg_gradient = None;
        let mut pos_gradient = None;
        if min_taste < 0.0 {
            neg_gradient = Some(
                colorgrad::CustomGradient::new()
                    .colors(&[
                        colorgrad::Color::from_rgba8(255, 0, 0, 255),
                        colorgrad::Color::from_rgba8(255, 255, 255, 64),
                    ])
                    .domain(&[min_taste, -0.0])
                    .build()
                    .unwrap(),
            );
        }
        if max_taste > 0.0 {
            pos_gradient = Some(
                colorgrad::CustomGradient::new()
                    .colors(&[
                        colorgrad::Color::from_rgba8(255, 255, 255, 64),
                        colorgrad::Color::from_rgba8(0, 255, 0, 255),
                    ])
                    .domain(&[0.0, max_taste])
                    .build()
                    .unwrap(),
            );
        }
        ColorGradient {
            neg_gradient,
            pos_gradient,
        }
    }

    pub fn for_taste(instrument: u32, attendees: &[Attendee]) -> Self {
        let min_taste = attendees
            .iter()
            .map(|as_| as_.tastes[instrument as usize] as i32)
            .min()
            .unwrap() as f64;
        let max_taste = attendees
            .iter()
            .map(|as_| as_.tastes[instrument as usize] as i32)
            .max()
            .unwrap() as f64;
        Self::new(min_taste, max_taste)
    }

    pub fn for_impact_map(grid: &Grid, impact_map: &ImpactMap) -> Self {
        let min = grid
            .positions
            .iter()
            .zip(&impact_map.scores)
            .filter(|(pos, _s)| !pos.taken)
            .map(|(_pos, s)| s.0)
            .min()
            .unwrap() as f64;
        let max = grid
            .positions
            .iter()
            .zip(&impact_map.scores)
            .filter(|(pos, _s)| !pos.taken)
            .map(|(_pos, s)| s.0)
            .max()
            .unwrap() as f64;
        Self::new(min, max)
    }

    pub fn get_color(&self, value: f64) -> raylib::prelude::Color {
        let gradient = if value < 0.0 {
            self.neg_gradient.as_ref().unwrap()
        } else {
            self.pos_gradient.as_ref().unwrap()
        };
        let c = gradient.at(value).to_rgba8();
        raylib::prelude::Color::new(c[0], c[1], c[2], c[3])
    }
}

pub fn gui_main(problem_path: &std::path::Path, solver_name: &str) {
    dbg!(problem_path);
    let problem = Problem::load(problem_path).expect("Failed to read the problem file");
    let data = &problem.data;

    let mut solver = create_solver(solver_name);
    solver.initialize(&problem);

    const WIDTH: i32 = 800;
    const HEIGHT: i32 = 800;
    const MARGIN: i32 = 20;
    const RIGHT_SIDE_WIDTH: i32 = 200;

    let (mut rl, thread) = raylib::init()
        .size(WIDTH + MARGIN * 2 + RIGHT_SIDE_WIDTH, HEIGHT + MARGIN * 2)
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

    let max_instrument = data.musicians.iter().map(|i| i.0).max().unwrap();

    let mut solution: Option<SolutionDto> = None;
    let mut score = None;
    let mut auto_step = false;
    let mut auto_score = false;
    let mut done = false;
    let mut selected_instrument = None;
    let mut taste_gradient = None;

    while !rl.window_should_close() {
        // ===== HIT TEST =====
        // TODO

        // ===== INTERACTION =====
        let mut do_step = auto_step;
        if let Some(k) = rl.get_key_pressed() {
            match k {
                KeyboardKey::KEY_SPACE => {
                    if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                        auto_step = !auto_step;
                    }
                    do_step = true;
                }
                KeyboardKey::KEY_S => {
                    if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                        auto_score = !auto_score;
                    }
                    if let Some(solution) = solution.as_ref() {
                        score = Some(crate::scorer::score(&data, &solution.placements));
                    }
                }
                KeyboardKey::KEY_Q => {
                    if selected_instrument == Some(0) {
                        selected_instrument = None;
                    } else if selected_instrument.is_none() {
                        selected_instrument = Some(max_instrument);
                    } else {
                        selected_instrument = Some(selected_instrument.unwrap() - 1);
                    }
                    if let Some(instrument) = selected_instrument {
                        taste_gradient =
                            Some(ColorGradient::for_taste(instrument, &data.attendees));
                    } else {
                        taste_gradient = None;
                    }
                }
                KeyboardKey::KEY_W => {
                    if selected_instrument.is_none() {
                        selected_instrument = Some(0);
                    } else if selected_instrument == Some(max_instrument) {
                        selected_instrument = None;
                    } else {
                        selected_instrument = Some(selected_instrument.unwrap() + 1);
                    }
                    if let Some(instrument) = selected_instrument {
                        taste_gradient =
                            Some(ColorGradient::for_taste(instrument, &data.attendees));
                    } else {
                        taste_gradient = None;
                    }
                }
                _ => {}
            }
        }

        // ===== HANDLING =====

        if do_step {
            if !done {
                let (s, d) = solver.solve_step();
                if auto_score {
                    score = Some(crate::scorer::score(&data, &s.placements));
                } else {
                    score = None;
                }
                Solution {
                    data: s.clone(),
                    score: score.unwrap_or(Score(0)),
                }
                .save(
                    solver.name().to_owned(),
                    &problem,
                    &PathBuf::from("./solutions/current/gui"),
                )
                .expect("Failed to write solution");
                solution = Some(s);
                done = d;
            }
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
                taste_gradient
                    .as_ref()
                    .map(|g| {
                        g.get_color(attendee.tastes[selected_instrument.unwrap() as usize] as f64)
                    })
                    .unwrap_or(Color::DARKBROWN),
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
            for p in &solution.placements {
                if !p.x.is_nan() {
                    d.draw_circle(
                        MARGIN + (p.x * ratio) as i32,
                        MARGIN + (p.y * ratio) as i32,
                        5.0 * ratio,
                        Color::BLACK,
                    );
                }
            }
        }

        if let Some(instrument) = selected_instrument {
            if let Some(impact_map) = solver.get_impact_map(&Instrument(instrument)) {
                let grid = solver.get_grid().unwrap();
                let gradient = ColorGradient::for_impact_map(grid, impact_map);
                for (pos, score) in grid.positions.iter().zip(&impact_map.scores) {
                    if !pos.taken {
                        d.draw_rectangle(
                            MARGIN + ((pos.p.x - 1.0) * ratio) as i32,
                            MARGIN + ((pos.p.y - 1.0) * ratio) as i32,
                            (3.0 * ratio) as i32,
                            (3.0 * ratio) as i32,
                            gradient.get_color(score.0 as f64),
                        );
                    }
                }
            }
        }

        // Right side

        let lines = &[
            format!(
                "Done: {}",
                if done { "true" } else { "false <press Space>" }
            ),
            format!(
                "Current score: {}",
                if score.is_none() {
                    "<press [Shift+]S>".to_owned()
                } else {
                    score.unwrap().0.to_string()
                }
            ),
            format!(
                "Focused instrument: {}",
                if selected_instrument.is_none() {
                    "<press Q/W>".to_owned()
                } else {
                    selected_instrument.unwrap().to_string()
                }
            ),
        ];

        for (idx, line) in lines.into_iter().enumerate() {
            d.draw_text(
                line,
                WIDTH + MARGIN * 2,
                MARGIN + 12 * idx as i32,
                12,
                Color::BLACK,
            );
        }
    }
}
