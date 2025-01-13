use std::ops::Deref;
use std::thread;
use std::time::{Duration, SystemTime};
use quarve::core::slock_owner;
use quarve::prelude::*;
use quarve::state::{Binding, Filterless, JoinedSignal, Store, WeakBinding};
use quarve::state::SetAction::Set;
use quarve::view::color_view::EmptyView;
use quarve::view::control::Dropdown;
use quarve::view::modal::{MessageBox, MessageBoxButton};
use quarve::view::scroll::ScrollView;
use quarve::view::text::{Text, TextField, TextModifier};
use quarve::view_match;
use crate::IVP;
use crate::parser::begin_parse;
use crate::scoreboard::{ProblemStatus, Scoreboard, TeamResult, ScoreboardOption};

#[allow(unused)]
fn dummy_scoreboard() -> Scoreboard {
    Scoreboard {
        num_problems: 10,
        entries: vec![
            TeamResult {
                team: "UCSD 1".to_string(),
                problems: vec![
                    ProblemStatus::Incorrect,
                    ProblemStatus::Incorrect,
                    ProblemStatus::Solved {
                        attempts: 2,
                        minutes: 301,
                        first_solve: false,
                    }
                ],
            },
            TeamResult {
                team: "UCSD 2".to_string(),
                problems: vec![
                    ProblemStatus::Solved {
                        attempts: 3,
                        minutes: 140,
                        first_solve: false,
                    },
                    ProblemStatus::Solved {
                        attempts: 1,
                        minutes: 300,
                        first_solve: true,
                    }
                ],
            }
        ],
    }
}

fn divider() -> impl IVP {
    LIGHT_GRAY
        .frame(F.intrinsic(1,1).unlimited_width())
}

pub fn viewer() -> impl IVP {
    let contest_type = Store::new(None);
    let url = Store::new("".to_string());
    let contest_data = Store::new(ScoreboardOption::None);
    
    vstack()
        .push(
            text("ICPC Live Scoreboard")
                .text_size(36)
                .padding(10)
        )
        .push(selector(contest_type.binding(), url.binding(), contest_data.binding()))
        .push(divider())
        .push(main_content(contest_data.binding()))
        .frame(F.unlimited_stretch())
        .text_color(WHITE)
        .bg_color(BLACK)
}

fn selector(
    contest_type: impl Binding<Filterless<Option<String>>> + Clone,
    url: impl Binding<Filterless<String>> + Clone,
    contest_data: impl Binding<Filterless<ScoreboardOption>> + Clone,
) -> impl IVP {

    hstack()
        .push(
            text("Contest Type")
                .bold()
        )
        .push(
            Dropdown::new(contest_type.clone())
                .option("NAC")
                .option("CERC")
                .intrinsic(100, 22)
        )
        .push(
            text("Scoreboard URL:")
                .bold()
        )
        .push(
            TextField::new(url.clone())
                .unstyled()
                .padding(2)
                .layer(L.border(LIGHT_GRAY, 1).radius(2))
                .intrinsic(300, 28)
        )
        .push(
            button("Go", move |s| {
                match contest_type.borrow(s).deref() {
                    Some(ref content) => {
                        let content = content.clone();
                        let url = url.clone();
                        let contest_data = contest_data.clone();
                        tokio::spawn(async move {
                            let url = {
                                let s = slock_owner();
                                let res = url.borrow(s.marker()).clone();
                                res
                            };

                            begin_parse(&content, &url, contest_data).await
                        });
                    }
                    None => {
                        MessageBox::new("Invalid".into(), "Select a contest type".into())
                            .button(MessageBoxButton::Ok)
                            .run(|_, _| { });
                    }
                }
            })
                .text_color(BLUE)
        )
        .padding(5)
}

fn scoreboard(sb: &Scoreboard) -> impl IVP {
    // timer controls
    let timer = Store::new(0);

    let binding = timer.weak_binding();
    thread::spawn(move || {
        let start = SystemTime::now();
        loop {
            thread::sleep(Duration::from_secs(60));
            let Some(binding) = binding.upgrade() else {
                break;
            };

            let s = slock_owner();
            let duration = SystemTime::now()
                .duration_since(start).unwrap();
            binding.apply(Set((duration.as_secs() as usize).min(300 * 60)), s.marker());
        }
    });

    let timer_sig = timer.signal();
    let controls =
        ivp_using(move |_, s| {
            hstack()
                .push(
                    Text::from_signal(timer_sig.map(|time| {
                        let raw_minutes = *time / 60;
                        let minutes = raw_minutes % 60;
                        let hours = raw_minutes / 60;

                        format!("Time {:0>2}:{:0>2}", hours, minutes)
                    }, s))
                        .padding(5)
                        .frame(F.intrinsic(90, 30).align(Alignment::Leading) )
                        .border(LIGHT_GRAY, 1)
                        .padding_edge(5, edge::DOWN | edge::LEFT)
                )
        });

    let sb = sb.clone();

    // problem headers
    let problems = (0..sb.num_problems)
        .into_iter()
        .hmap_options(|i, _s| {
            text(&"ABCDEFGHIJKLMNOPQRSTUVWXYZ"[*i ..*i + 1])
                .intrinsic(22, 22)
                .bold()
                .padding(3)
                .layer(L.radius(2).border(DARK_GRAY, 1))
                .intrinsic(50, 30)
        }, HStackOptions::default().spacing(0.0));

    let team_results = sb.entries;

    let items = ivp_using(move |_, s| {
        let sorted_items = timer.signal()
            .map(move |time| {
                let elapsed_minutes = *time / 60;
                let mut team_results = team_results.clone();
                team_results.sort_by(|i1, i2| {
                    i1.score(elapsed_minutes).cmp(&i2.score(elapsed_minutes))
                });
                team_results
                    .into_iter().enumerate()
                    .collect::<Vec<(usize, TeamResult)>>()
            }, s);

        sorted_items
            .sig_vmap_options(move |(index, se), s| {
                let se2 = se.clone();
                let solved_time = timer.signal().map(move |time| {
                    let res = se2.score(*time / 60);
                    res
                }, s);
                let solved = solved_time.map(|(s, _)| (-s).to_string(), s);
                let time = solved_time.map(|(_, t)| t.to_string(), s);
                let score =
                    HStack::hetero_options(
                        HStackOptions::default()
                            .spacing(1.0)
                    )
                        .push(
                            Text::from_signal(solved)
                                .bold()
                        )
                        .push(
                            Text::from_signal(time)
                                .text_color(DARK_GRAY)
                                .text_size(10)
                        )
                        .intrinsic(58, 38)
                        .border(LIGHT_GRAY, 1)
                        .intrinsic(60, 40);

                let timer_sig = timer.signal();
                let solves = se.problems.clone()
                    .hmap_options(move |solve, s| {
                        let signal = JoinedSignal::join_map(
                            &timer_sig, &FixedSignal::new(solve.clone()),
                            |u, v| {
                                let elapsed_minutes = *u / 60;
                                match v {
                                    ProblemStatus::Incorrect => ProblemStatus::Incorrect,
                                    ProblemStatus::Solved {
                                        minutes, ..
                                    } => {
                                        if elapsed_minutes < *minutes {
                                            ProblemStatus::Incorrect
                                        } else {
                                            v.clone()
                                        }
                                    }
                                }
                            }, s
                        );

                        view_match!(signal;
                            ProblemStatus::Incorrect => {
                                CLEAR
                                    .intrinsic(50, 40)
                            },
                            ProblemStatus::Solved { attempts, minutes, first_solve } => {
                                let color = if *first_solve {
                                    rgb(32, 159, 23)
                                } else {
                                    rgb(84, 231, 77)
                                };

                                VStack::hetero_options(
                                    VStackOptions::default()
                                    .spacing(4.0)
                                )
                                    .push(
                                        text(minutes.to_string())
                                    )
                                    .push(
                                        text(attempts.to_string() + if *attempts == 1 { " try" } else { " tries"})
                                        .text_size(10)
                                    )
                                    .intrinsic(50, 40)
                                    .bg_color(color)
                            }
                        )
                    }, HStackOptions::default().spacing(0.0))
                    .text_color(BLACK);

                VStack::hetero_options(VStackOptions::default()
                    .align(HorizontalAlignment::Leading)
                    .spacing(0.0)
                )
                    .push(
                        hstack()
                            .push(
                                text((index + 1).to_string())
                                    .intrinsic(50, 30)
                            )
                            .push(
                                text(se.team.clone())
                                    .intrinsic(200, 30)
                                    .bold()
                            )
                            .push(score)
                            .push(solves)
                    )
                    .push(divider())
            }, VStackOptions::default().spacing(0.0))
    });

    VStack::hetero_options(
        VStackOptions::default()
            .align(HorizontalAlignment::Leading)
            .spacing(0.0)
    )
        .push(controls)
        .push(
            hstack()
                .push(
                    EmptyView.intrinsic(50, 30)
                )
                .push(
                    text("Team")
                        .bold()
                        .text_color(RED)
                        .intrinsic(200, 30)
                )
                .push(
                    text("Score")
                        .bold()
                        .text_color(BLUE)
                        .intrinsic(60, 30)
                )
                .push(problems)
                .padding_edge(10, edge::DOWN)
        )
        .push(divider())
        .push(
            ScrollView::vertical(
                vstack()
                    .push(items)
            )
        )
}

fn main_content(
    contest_data: impl Binding<Filterless<ScoreboardOption>> + Clone,
) -> impl IVP {

    view_match!(contest_data;
        ScoreboardOption::Some(sb) => {
            scoreboard(sb)
        },
        ScoreboardOption::None => {
            text("Select a contest")
                .frame(F.unlimited_stretch())
        }
    )
        .bg_color(WHITE)
        .text_color(BLACK)
}