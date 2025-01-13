use quarve::core::slock_owner;
use quarve::state::{Binding, Filterless};
use quarve::state::SetAction::Set;
use quarve::view::modal::{MessageBox, MessageBoxButton};
use crate::scoreboard::{ScoreboardOption};

mod nac;
mod cerc;
mod util;

pub async fn begin_parse(contest_type: &str, url: &str, result: impl Binding<Filterless<ScoreboardOption>>) {
    match contest_type {
        "NAC" => {
              todo!("Not implemented")
        },
        "CERC" => {
            match cerc::fetch_and_parse_scoreboard(url).await {
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
                        .run(|_, _| { })
                }
            }
        }
        &_ => {
            unreachable!()
        }
    };
}