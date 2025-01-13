use reqwest;
use scraper::{Html, Selector};
use std::error::Error;
use crate::parser::util::add_first_solves;
use crate::scoreboard::{ProblemStatus, Scoreboard, TeamResult};

pub async fn fetch_and_parse_scoreboard(url: &str) -> Result<Scoreboard, Box<dyn Error>> {
    // Fetch the HTML content
    let html_content = reqwest::get(url).await?.text().await?;

    // Parse the HTML
    let document = Html::parse_document(&html_content);

    // Selectors for parsing
    let table_selector = Selector::parse("table.scoreboard").unwrap();
    let team_row_selector = Selector::parse("tr[data-ajax-id]").unwrap();
    let team_name_selector = Selector::parse(".team-name .single-line").unwrap();
    let task_selector = Selector::parse(".task").unwrap();
    let tries_selector = Selector::parse(".tries").unwrap();
    let penalty_time_selector = Selector::parse(".penalty-time").unwrap();

    let mut entries = Vec::new();
    let mut num_problems = 0;

    // Locate the scoreboard table
    if let Some(scoreboard_table) = document.select(&table_selector).next() {
        // Iterate over each team row within the table
        for team_row in scoreboard_table.select(&team_row_selector) {
            // Extract the team name
            let team_name = team_row
                .select(&team_name_selector)
                .next()
                .map(|el| el.inner_html().trim().to_string())
                .unwrap_or_else(|| "Unknown Team".to_string());

            let mut problems = Vec::new();

            // Iterate over each problem cell
            for task in team_row.select(&task_selector) {
                let task_class = task.value().attr("class").unwrap_or("");

                if task_class.contains("solved") {
                    let attempts = task
                        .select(&tries_selector)
                        .next()
                        .and_then(|el| el.inner_html().trim().parse::<usize>().ok())
                        .unwrap_or(0);

                    let penalty_time = task
                        .select(&penalty_time_selector)
                        .next()
                        .and_then(|el| parse_time_to_minutes(&el.inner_html().trim(), attempts))
                        .unwrap_or(0);

                    let first_solve = false; // Assuming first_solve isn't in the data provided

                    problems.push(ProblemStatus::Solved {
                        attempts,
                        minutes: penalty_time,
                        first_solve,
                    });
                } else {
                    problems.push(ProblemStatus::Incorrect);
                }
            }

            num_problems = num_problems.max(problems.len());
            entries.push(TeamResult { team: team_name, problems });
        }
    }

    let mut res = Scoreboard {
        num_problems,
        entries,
    };
    add_first_solves(&mut res);

    Ok(res)
}

fn parse_time_to_minutes(time_str: &str, attempts: usize) -> Option<usize> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 3 {
        let hours = parts[0].parse::<usize>().ok()?;
        let minutes = parts[1].parse::<usize>().ok()?;
        let seconds = parts[2].parse::<usize>().ok()?;
        let offset = (attempts - 1) * 20;
        Some(hours * 60 + minutes + (seconds / 60) - offset)
    } else {
        None
    }
}
