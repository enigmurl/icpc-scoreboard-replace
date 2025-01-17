use std::error::Error;
use quarve::core::slock_owner;
use quarve::state::{Binding, Filterless};
use quarve::state::SetAction::Set;
use quarve::view::modal::{MessageBox, MessageBoxButton};
use crate::scoreboard::{Scoreboard, ScoreboardOption};

mod kattis;
mod cerc;
mod util;
mod asia_jakarta;
mod nerc;

pub fn handle(f: Result<Scoreboard, Box<dyn Error>>, result: impl Binding<Filterless<ScoreboardOption>>) {
    match f {
        Ok(scoreboard) => {
            let s = slock_owner();
            result.apply(Set(ScoreboardOption::Some(scoreboard)), s.marker());
        }
        Err(e) => {
            MessageBox::new(
                Some("Operation Failed"),
                Some(&e.to_string())
            )
                .button(MessageBoxButton::Ok)
                .run(|_, _| {})
        }
    }
}
pub async fn begin_parse(contest_type: &str, url: &str, result: impl Binding<Filterless<ScoreboardOption>>) {
    match contest_type {
        "KATTIS" => {
            handle(kattis::fetch_and_parse_scoreboard(url).await, result);
        },
        "CERC" => {
            handle(cerc::fetch_and_parse_scoreboard(url).await, result);
        }
        &_ => {
            unreachable!()
        }
    };
}