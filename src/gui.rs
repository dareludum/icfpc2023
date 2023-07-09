use std::path::PathBuf;

use colorgrad::Gradient;
use raylib::prelude::*;

use crate::{
    common::{calculate_invalid_positions, Grid},
    dto::{Attendee, Instrument, SolutionDto},
    geometry::{distance2, Coords2D},
    scoring::impact_map::ImpactMap,
    solvers::{create_solver, Problem, Solution, Solver},
};

struct ColorGradient {
    neg_gradient: Option<Gradient>,
    pos_gradient: Option<Gradient>,
}

impl Coords2D for Vector2 {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

impl ColorGradient {
    pub fn new(min_taste: f64, max_taste: f64) -> Self {
        let mut neg_gradient = None;
        let mut pos_gradient = None;
        if min_taste <= 0.0 {
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
        let gradient = if value <= 0.0 {
            self.neg_gradient.as_ref().unwrap()
        } else {
            self.pos_gradient.as_ref().unwrap()
        };
        let c = gradient.at(value).to_rgba8();
        raylib::prelude::Color::new(c[0], c[1], c[2], c[3])
    }
}

const WIDTH: i32 = 800;
const HEIGHT: i32 = 800;
const MARGIN: i32 = 20;
const RIGHT_SIDE_WIDTH: i32 = 200;

#[derive(Default)]
struct State {
    problem: Problem,
    solution: Solution,
    solver: Option<Box<dyn Solver>>,

    ratio: f32,
    max_instrument: u32,
    auto_step: bool,
    done: bool,
    show_pruned_data: bool,
    selected_instrument: Option<u32>,
    selected_musician: Option<usize>,
    dragged_musician: Option<(usize, Vector2)>,
    did_drag_musician: bool,
    taste_gradient: Option<ColorGradient>,
    drag_start: Option<Vector2>,
    viewport_offset: Vector2,
    viewport_zoom_level: i32,
    viewport_zoom: f32,
}

impl State {
    fn new(problem_path: &std::path::Path) -> Self {
        let problem = Problem::load(problem_path).expect("Failed to read the problem file");
        let data = &problem.data;

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

        let max_instrument = data.musicians.iter().map(|i| i.0).max().unwrap();

        State {
            problem,
            ratio,
            max_instrument,
            show_pruned_data: true,
            viewport_zoom: 1.0,
            ..State::default()
        }
    }

    pub fn update_score(&mut self) {
        self.solution.score = crate::scoring::new_scorer::new_score(
            &self.problem.data,
            &self.solution.data.placements,
        );
    }

    pub fn save_solution(&self) {
        let name = if let Some(solver) = self.solver.as_ref() {
            solver.name()
        } else {
            "gui".to_owned()
        };
        self.solution
            .save(
                name,
                &self.problem,
                &PathBuf::from("./solutions/current/gui"),
            )
            .expect("Failed to write solution");
    }
}

pub fn gui_main(problem_path: &std::path::Path, solver_name: &str) {
    std::fs::create_dir_all("./solutions/current/gui")
        .expect("Failed to create the directory ./solutions/current/gui");
    logging::set_trace_log(TraceLogLevel::LOG_WARNING);

    let (mut rl, thread) = raylib::init()
        .size(WIDTH + MARGIN * 2 + RIGHT_SIDE_WIDTH, HEIGHT + MARGIN * 2)
        .title(&format!("ICFPC2023 - Dare Ludum - {:#?}", problem_path))
        .build();

    let mut state = State::new(problem_path);

    'main: while !rl.window_should_close() {
        // ===== HIT TEST =====
        #[derive(Debug)]
        enum HitTestResult {
            Empty,
            Musician(usize),
        }
        let mouse_pos_logical = (rl.get_mouse_position()
            - state.viewport_offset
            - Vector2::new(MARGIN as f32, MARGIN as f32))
        .scale_by(1.0 / (state.ratio * state.viewport_zoom));

        let mut hit_test_result = HitTestResult::Empty;
        let mut musicians_in_range = state
            .solution
            .data
            .placements
            .iter()
            .enumerate()
            .filter(|(_idx, pos)| !pos.x.is_nan())
            .map(|(idx, pos)| (idx, distance2(pos, &mouse_pos_logical)))
            .filter(|(_idx, dist2)| *dist2 <= 100.0)
            .collect::<Vec<_>>();
        musicians_in_range.sort_by(|(_idx0, dist0), (_idx1, dist1)| dist0.total_cmp(dist1));
        if !musicians_in_range.is_empty() {
            hit_test_result = HitTestResult::Musician(musicians_in_range[0].0);
        }

        // ===== INTERACTION =====

        let mut do_step = state.auto_step;
        if let Some(k) = rl.get_key_pressed() {
            match k {
                KeyboardKey::KEY_SPACE => {
                    if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                        state.auto_step = !state.auto_step;
                    }
                    do_step = true;
                }
                KeyboardKey::KEY_Q => {
                    if state.selected_instrument == Some(0) {
                        state.selected_instrument = None;
                    } else if state.selected_instrument.is_none() {
                        state.selected_instrument = Some(state.max_instrument);
                    } else {
                        state.selected_instrument = Some(state.selected_instrument.unwrap() - 1);
                    }
                    if let Some(instrument) = state.selected_instrument {
                        state.taste_gradient = Some(ColorGradient::for_taste(
                            instrument,
                            &state.problem.data.attendees,
                        ));
                    } else {
                        state.taste_gradient = None;
                    }
                }
                KeyboardKey::KEY_W => {
                    if state.selected_instrument.is_none() {
                        state.selected_instrument = Some(0);
                    } else if state.selected_instrument == Some(state.max_instrument) {
                        state.selected_instrument = None;
                    } else {
                        state.selected_instrument = Some(state.selected_instrument.unwrap() + 1);
                    }
                    if let Some(instrument) = state.selected_instrument {
                        state.taste_gradient = Some(ColorGradient::for_taste(
                            instrument,
                            &state.problem.data.attendees,
                        ));
                    } else {
                        state.taste_gradient = None;
                    }
                }
                KeyboardKey::KEY_E => {
                    state.show_pruned_data = !state.show_pruned_data;
                }
                KeyboardKey::KEY_LEFT_BRACKET | KeyboardKey::KEY_RIGHT_BRACKET => {
                    let current_problem_id = state.problem.id.parse::<i16>().unwrap();
                    let new_problem_id = if k == KeyboardKey::KEY_LEFT_BRACKET {
                        current_problem_id - 1
                    } else {
                        current_problem_id + 1
                    };
                    let mut new_problem_path = problem_path.to_owned();
                    new_problem_path.set_file_name(&format!("{}.json", new_problem_id));
                    if new_problem_path.exists() {
                        state = State::new(&new_problem_path);
                        rl.set_window_title(
                            &thread,
                            &format!("ICFPC2023 - Dare Ludum - {:#?}", new_problem_path),
                        );
                        continue 'main;
                    }
                }
                _ => {}
            }
        }

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
            if let HitTestResult::Musician(idx) = hit_test_result {
                if let Some(selected_idx) = state.selected_musician {
                    state.solution.data.placements.swap(idx, selected_idx);
                    state.update_score();
                    state.save_solution();
                    state.selected_musician = None;
                } else {
                    state.selected_musician = Some(idx);
                    state.dragged_musician = Some((idx, mouse_pos_logical));
                }
            }
        } else if rl.is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON) {
            state.dragged_musician = None;
            if state.did_drag_musician {
                state.selected_musician = None;
                state.did_drag_musician = false;
            }
        }
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_MIDDLE_BUTTON) {
            state.drag_start = Some(rl.get_mouse_position());
        } else if rl.is_mouse_button_released(MouseButton::MOUSE_MIDDLE_BUTTON) {
            state.drag_start = None;
        }

        let wheel_move = rl.get_mouse_wheel_move() as i32;
        if wheel_move != 0 {
            // TODO: Doesn't work well, fix if there's time
            state.viewport_zoom_level += wheel_move;
            state.viewport_zoom = 1.0 + 0.5 * state.viewport_zoom_level as f32;
            state.viewport_offset -= rl.get_mouse_position().scale_by(wheel_move as f32 * 0.5);
        }

        // ===== HANDLING =====

        if let Some(start) = state.drag_start {
            let new_mouse_pos = rl.get_mouse_position();
            let offset = start - new_mouse_pos;
            state.viewport_offset -= offset;
            state.drag_start = Some(new_mouse_pos);
        }
        if let Some((idx_pos, start)) = state.dragged_musician {
            if start != mouse_pos_logical {
                let offset = start - mouse_pos_logical;
                let pos = &mut state.solution.data.placements[idx_pos];
                pos.x -= offset.x;
                pos.y -= offset.y;
                state.update_score();
                state.save_solution();
                state.dragged_musician = Some((idx_pos, mouse_pos_logical));
                state.did_drag_musician = true;
            }
        }

        if do_step && !state.done {
            if state.solver.is_none() {
                let mut solver = create_solver(solver_name);
                solver.initialize(&state.problem, SolutionDto::default());
                state.solver = Some(solver);
            }
            let (s, d) = state.solver.as_mut().unwrap().solve_step();
            state.solution.data = s;
            state.update_score();
            state.save_solution();
            state.done = d;
        }

        // ===== DRAWING =====

        let invalid_musician_positions =
            calculate_invalid_positions(&state.solution.data.placements, &state.problem.data);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::GRAY);

        let origin_x = state.viewport_offset.x as i32 + MARGIN;
        let origin_y = state.viewport_offset.y as i32 + MARGIN;
        let zoomed_ratio = state.ratio * state.viewport_zoom;

        let transform_x = |x: f32| origin_x + (x * zoomed_ratio) as i32;
        let transform_y = |y: f32| origin_y + (y * zoomed_ratio) as i32;
        let transform_size = |size: f32| (size * zoomed_ratio) as i32;

        d.draw_rectangle(
            transform_x(0.0),
            transform_y(0.0),
            transform_size(state.problem.data.room_width),
            transform_size(state.problem.data.room_height),
            Color::LIGHTGRAY,
        );

        d.draw_rectangle(
            transform_x(state.problem.data.stage_bottom_left.0),
            transform_y(state.problem.data.stage_bottom_left.1),
            transform_size(state.problem.data.stage_width),
            transform_size(state.problem.data.stage_height),
            Color::BEIGE,
        );

        for p in &state.problem.data.pillars {
            d.draw_circle(
                transform_x(p.center.0),
                transform_y(p.center.1),
                p.radius * zoomed_ratio,
                Color::GRAY,
            )
        }
        if state.show_pruned_data {
            for p in &state.problem.removed_pillars {
                d.draw_circle(
                    transform_x(p.center.0),
                    transform_y(p.center.1),
                    p.radius * zoomed_ratio,
                    Color::LIGHTSKYBLUE,
                )
            }
        }

        for attendee in state.problem.data.attendees.iter() {
            d.draw_circle(
                transform_x(attendee.x),
                transform_y(attendee.y),
                10.0 * zoomed_ratio,
                state
                    .taste_gradient
                    .as_ref()
                    .map(|g| {
                        g.get_color(
                            attendee.tastes[state.selected_instrument.unwrap() as usize] as f64,
                        )
                    })
                    .unwrap_or(Color::DARKBROWN),
            );
        }
        if state.show_pruned_data {
            for attendee in state.problem.removed_attendees.iter() {
                d.draw_circle(
                    transform_x(attendee.x),
                    transform_y(attendee.y),
                    10.0 * zoomed_ratio,
                    Color::LIGHTSKYBLUE,
                );
            }
        }

        for (idx, p) in state.solution.data.placements.iter().enumerate() {
            if !p.x.is_nan() && !invalid_musician_positions.contains(&idx) {
                d.draw_circle(
                    transform_x(p.x),
                    transform_y(p.y),
                    10.0 * zoomed_ratio,
                    Color::BLUE,
                );
            }
        }
        for (idx, p) in state.solution.data.placements.iter().enumerate() {
            if !p.x.is_nan() && invalid_musician_positions.contains(&idx) {
                d.draw_circle(
                    transform_x(p.x),
                    transform_y(p.y),
                    10.0 * zoomed_ratio,
                    Color::RED,
                );
            }
        }
        for (idx, p) in state.solution.data.placements.iter().enumerate() {
            if !p.x.is_nan() {
                d.draw_circle(
                    transform_x(p.x),
                    transform_y(p.y),
                    5.0 * zoomed_ratio,
                    if invalid_musician_positions.contains(&idx) {
                        Color::DARKRED
                    } else {
                        Color::BLACK
                    },
                );
            }
        }
        if zoomed_ratio >= 1.0 {
            for (idx, p) in state.solution.data.placements.iter().enumerate() {
                if !p.x.is_nan() {
                    d.draw_text(
                        &state.problem.data.musicians[idx].0.to_string(),
                        transform_x(p.x) - 2,
                        transform_y(p.y) - 4,
                        10,
                        Color::WHITE,
                    )
                }
            }
        }

        // Post-clear the right panel
        d.draw_rectangle(
            MARGIN * 2 + WIDTH,
            0,
            RIGHT_SIDE_WIDTH,
            HEIGHT + MARGIN * 2,
            Color::GRAY,
        );

        if let Some(instrument) = state.selected_instrument {
            let impact_map = if let Some(solver) = state.solver.as_ref() {
                solver.get_impact_map(&Instrument(instrument))
            } else {
                None
            };
            if let Some(impact_map) = impact_map {
                let grid = state.solver.as_ref().unwrap().get_grid().unwrap();
                let gradient = ColorGradient::for_impact_map(grid, impact_map);
                for (pos, score) in grid.positions.iter().zip(&impact_map.scores) {
                    if !pos.taken {
                        d.draw_rectangle(
                            transform_x(pos.p.x - 1.0),
                            transform_y(pos.p.y - 1.0),
                            transform_size(3.0),
                            transform_size(3.0),
                            gradient.get_color(score.0 as f64),
                        );
                    }
                }
            }
        }

        // Right side

        let lines = &[
            "Commands:".to_owned(),
            "  - Pan: Middle mouse drag".to_owned(),
            "  - Zoom: Mouse wheel".to_owned(),
            "  - Solve step: Space".to_owned(),
            "  - Solve (auto): Shift+Space".to_owned(),
            "  - Show/Hide pruned data: E".to_owned(),
            "  - Prev/Next instrument: Q/W".to_owned(),
            "  - Prev/Next problem: [/]".to_owned(),
            "".to_owned(),
            format!("Done: {}", state.done),
            format!("Current score: {}", state.solution.score.0),
            format!(
                "Focused instrument: {}",
                if state.selected_instrument.is_none() {
                    "<none>".to_owned()
                } else {
                    state.selected_instrument.unwrap().to_string()
                }
            ),
        ];

        for (idx, line) in lines.iter().enumerate() {
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
