use std::ops::Deref;
use quarve::state::{Binding, Filterless};
use crate::scoreboard::ScoreboardOption;

mod nac;
mod cerc;

pub fn begin_parse(contest_type: &str, result: impl Binding<Filterless<ScoreboardOption>>) {
    match contest_type {
        "NAC" => {

        },
        "CERC" => {

        }
        &_ => {
            unreachable!()
        }
    }
}