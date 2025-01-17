use reqwest;
use scraper::{Element, Html, Selector};
use std::error::Error;
use scraper::CaseSensitivity::AsciiCaseInsensitive;
use scraper::selector::CssLocalName;
use crate::scoreboard::{ProblemStatus, Scoreboard, TeamResult};

pub async fn fetch_and_parse_scoreboard(url: &str) -> Result<Scoreboard, Box<dyn Error>> {
    let html_content = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&html_content);

    // Updated selectors based on the new HTML structure
    let table_selector = Selector::parse(".standings-table").unwrap();
    let team_row_selector = Selector::parse("tbody tr").unwrap();
    let team_name_selector = Selector::parse(".standings-cell--expand a").unwrap();
    let problem_cell_selector = Selector::parse("td.solved, td.attempted, td.first").unwrap();
    let result_cell_text_selector = Selector::parse(".standings-table-result-cell-text").unwrap();
    let time_selector = Selector::parse(".standings-table-result-cell-time").unwrap();

    let mut entries = Vec::new();
    let mut num_problems = 0;

    if let Some(scoreboard_table) = document.select(&table_selector).next() {
        for team_row in scoreboard_table.select(&team_row_selector) {
            // Extract team name
            let team_name = team_row
                .select(&team_name_selector)
                .next()
                .map(|el| el.inner_html().trim().to_string())
                .unwrap_or_else(|| "Unknown Team".to_string());

            let mut problems = Vec::new();

            // Process each problem cell
            for problem_cell in team_row.select(&problem_cell_selector) {
                let cell_text = problem_cell
                    .select(&result_cell_text_selector)
                    .next()
                    .map(|el| el.text().collect::<String>())
                    .unwrap();

                let status = if problem_cell.has_class(&CssLocalName::from("solved"), AsciiCaseInsensitive)
                    || problem_cell.has_class(&CssLocalName::from("first"), AsciiCaseInsensitive) {
                    // Extract attempts and time
                    let attempts = cell_text
                        .trim()
                        .lines()
                        .next()
                        .and_then(|s| s.trim().parse::<usize>().ok())
                        .unwrap_or(1);

                    let minutes = problem_cell
                        .select(&time_selector)
                        .next()
                        .and_then(|el| parse_time_str(&el.inner_html().trim()))
                        .unwrap_or(0);

                    let first_solve = problem_cell.has_class(&CssLocalName::from("first"), AsciiCaseInsensitive);

                    ProblemStatus::Solved {
                        attempts,
                        minutes,
                        first_solve,
                    }
                } else if problem_cell.has_class(&CssLocalName::from("attempted"), AsciiCaseInsensitive) {
                    ProblemStatus::Incorrect
                } else {
                    ProblemStatus::Incorrect
                };

                problems.push(status);
            }

            num_problems = num_problems.max(problems.len());
            entries.push(TeamResult { team: team_name, problems });
        }
    }

    let ret = Scoreboard {
        num_problems,
        entries,
    };

    Ok(ret)
}

fn parse_time_str(time_str: &str) -> Option<usize> {
    // Handle format like "52 min"
    if time_str.ends_with(" min") {
        return time_str
            .trim_end_matches(" min")
            .parse::<usize>()
            .ok();
    }
    None
}