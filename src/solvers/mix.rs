use log::debug;

use crate::{dto::SolutionDto, scoring::new_scorer};

use super::{Problem, Solver};

#[derive(Default, Clone)]
pub struct Mix {
    problem: Problem,
    solution: SolutionDto,
}

impl Solver for Mix {
    fn name(&self) -> String {
        "mix".to_owned()
    }

    fn initialize(&mut self, problem: &Problem, solution: SolutionDto) {
        assert!(
            !solution.placements.is_empty(),
            "mix({}): must not be the start of the chain",
            problem.id
        );
        self.problem = problem.clone();
        self.solution = solution;
    }

    fn solve_step(&mut self) -> (SolutionDto, bool) {
        const VOLUME_MIN: f32 = 0.0;
        const VOLUME_MAX: f32 = 10.0;
        const VOLUME_DEFAULT: f32 = 1.0;
        const VOLUME_STEP: f32 = 0.5;

        let mut volumes = vec![VOLUME_DEFAULT; self.solution.placements.len()];
        let orig_score = new_scorer::new_score(
            &self.problem.data,
            &self.solution.placements,
            Some(&volumes),
        );
        for idx in 0..self.solution.placements.len() {
            let mut volume = VOLUME_MIN;
            while volume < VOLUME_MAX {
                let old_volume = volumes[idx];
                volumes[idx] = volume;
                let new_score = new_scorer::new_score(
                    &self.problem.data,
                    &self.solution.placements,
                    Some(&volumes),
                );
                if new_score.0 < orig_score.0 {
                    volumes[idx] = old_volume;
                }
                volume += VOLUME_STEP;
            }
            volume = VOLUME_MAX;
            // Copy-paste but whatever
            let old_volume = volumes[idx];
            volumes[idx] = volume;
            let new_score = new_scorer::new_score(
                &self.problem.data,
                &self.solution.placements,
                Some(&volumes),
            );
            if new_score.0 < orig_score.0 {
                volumes[idx] = old_volume;
            }
            debug!(
                "mix({}): {} musicians left",
                self.problem.id,
                self.solution.placements.len() - idx - 1
            );
        }

        debug!(
            "mix({}): {} full volume, {} silent, {} others",
            self.problem.id,
            volumes.iter().filter(|&&v| v == VOLUME_MAX).count(),
            volumes.iter().filter(|&&v| v == VOLUME_MIN).count(),
            volumes
                .iter()
                .filter(|&&v| v != VOLUME_MIN && v != VOLUME_MAX)
                .count(),
        );

        // dbg!(&volumes);

        (
            SolutionDto {
                placements: self.solution.placements.clone(),
                volumes: Some(volumes),
            },
            true,
        )
    }
}
