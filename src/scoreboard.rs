use quarve::state::{SetAction, Stateful};
use quarve::util::marker::FalseMarker;

 // for a given team
#[derive(Clone, PartialEq, Debug)]
pub enum ProblemStatus {
    Incorrect,
    Solved {
        attempts: usize,
        minutes: usize,
        first_solve: bool,
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct TeamResult {
    pub team: String,
    pub problems: Vec<ProblemStatus>
}

impl TeamResult {
    pub fn score(&self, elapsed_minutes: usize) -> (isize, usize) {
        let (solved, time) = self.problems.iter()
            .fold((0, 0), |mut status, curr | {
                if let ProblemStatus::Solved {
                        attempts, minutes, ..
                    } = curr {
                    if *minutes <= elapsed_minutes {
                        status.0 += 1;
                        status.1 += minutes + (attempts - 1) * 20;
                    }
                }

                status
            });

        (-solved, time)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Scoreboard {
    pub num_problems: usize,
    pub entries: Vec<TeamResult>
}

// need a new typ
#[derive(Clone, PartialEq)]
pub enum ScoreboardOption {
    Some(Scoreboard),
    None
}

impl Stateful for ScoreboardOption {
    type Action = SetAction<ScoreboardOption>;
    type HasInnerStores = FalseMarker;
}