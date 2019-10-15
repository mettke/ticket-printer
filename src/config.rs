use crate::Result;
use failure::ResultExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub pdf: PDfDimension,
    pub printer: Option<Printer>,
    pub trello: Option<Trello>,
    pub jira: Option<Jira>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Trello {
    pub app_key: String,
    pub token: String,
    pub print_label: String,
    pub limit_to_boards: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Jira {
    pub host: String,
    pub user: String,
    pub token: String,
    pub print_label: String,
    pub limit_to_projects: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PDfDimension {
    pub height: f32,
    pub width: f32,
    pub margin: f32,

    pub title_lines: u32,
    pub title_seperator_margin: f32,
    pub qrcode_seperator_margin: f32,
    pub subtitle_size: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Printer {
    pub media: String,
    pub orientation: String,
    pub number_of_copies: u16,
    pub name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            trello: None,
            jira: None,
            pdf: PDfDimension {
                height: 62.0,
                width: 100.0,
                margin: 4.0,

                title_lines: 2,
                title_seperator_margin: 4.0,
                qrcode_seperator_margin: 4.0,
                subtitle_size: 4.0,
            },
            printer: None,
        }
    }
}

impl Config {
    pub fn service_available(&self) -> bool {
        self.trello.is_some() || self.jira.is_some()
    }
}

pub fn get() -> Result<Config> {
    Ok(confy::load("ticket_printer").with_context(|_| {
        "Could not load configuration".to_string()
    })?)
}
