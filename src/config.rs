use crate::Result;
use directories::BaseDirs;
use failure::ResultExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub pdf: PDfDimension,
    #[serde(default)]
    pub printer: Option<Printer>,
    #[serde(default)]
    pub trello: Option<Trello>,
    #[serde(default)]
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

impl Default for PDfDimension {
    fn default() -> Self {
        Self {
            height: 62.0,
            width: 100.0,
            margin: 4.0,

            title_lines: 2,
            title_seperator_margin: 4.0,
            qrcode_seperator_margin: 4.0,
            subtitle_size: 4.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Printer {
    pub media: String,
    pub orientation: String,
    pub number_of_copies: u16,
    pub name: String,
}

impl Config {
    pub fn service_available(&self) -> bool {
        self.trello.is_some() || self.jira.is_some()
    }
}

pub fn get() -> Result<Config> {
    let mut settings = config::Config::default();
    let _ = settings
        .merge(
            config::File::with_name(
                "/etc/ticket_printer/ticket_printer",
            )
            .required(false),
        )
        .expect("Configuration froozen 1");
    let home: Option<String> = BaseDirs::new().and_then(|dir| {
        dir.config_dir().to_str().map(|str| str.into())
    });
    if let Some(mut dir) = home {
        dir.push_str("/ticket_printer/ticket_printer");
        let _ = settings
            .merge(config::File::with_name(&dir).required(false))
            .expect("Configuration froozen 2");
    }
    let _ = settings
        .merge(
            config::File::with_name("ticket_printer")
                .required(false),
        )
        .expect("Configuration froozen 3");
    Ok(settings.try_into().with_context(|_| {
        "Could not load configuration".to_string()
    })?)
}
