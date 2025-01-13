use crate::scoreboard::{ProblemStatus, Scoreboard};

pub fn add_first_solves(sb: &mut Scoreboard) {
    for i in 0 .. sb.num_problems {
        let min = sb.entries.iter()
            .map(|e| match &e.problems[i] {
                ProblemStatus::Incorrect => { usize::MAX }
                ProblemStatus::Solved { minutes, .. } => {
                    *minutes
                }
            })
            .min();

        if let Some(m) = min {
            for p in sb.entries.iter_mut() {
                match &mut p.problems[i] {
                    ProblemStatus::Incorrect => {}
                    ProblemStatus::Solved { minutes, first_solve, .. } => {
                        if *minutes == m {
                            *first_solve = true;
                            break;
                        }
                    }
                }
            }
        }
    }
}