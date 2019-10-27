use crate::config;
use clap::{
    app_from_crate, crate_authors, crate_description, crate_name,
    crate_version, Arg,
};
use failure::ResultExt;
use std::env;

const GENERATE_CONFIG: &str = "generate-example-config";
const GENERATE_CONFIG_POSSIBLE: &[&str; 3] =
    &["json", "yaml", "toml"];
const POLL_SECS: &str = "poll";

pub struct Arguments {
    pub poll: Option<u64>,
}

pub fn handle() -> crate::Result<Arguments> {
    let matches = setup();
    if let Some(config_type) = matches.value_of(GENERATE_CONFIG) {
        let config = create_default_config();
        let config_text = stringify_config(config_type, &config)?;
        println!("{}", config_text);
    }
    let poll =
        matches.value_of(POLL_SECS).and_then(|s| s.parse().ok());
    Ok(Arguments { poll })
}

fn setup() -> clap::ArgMatches<'static> {
    app_from_crate!()
        .after_help("You can setup your configuration file in the following places:

    1. /etc/ticket_printer/ticket_printer.{ext}
    2. ~/.config/ticket_printer/ticket_printer.{ext}
    3. ticket_printer.{ext}

Configuration is merged from 1 to 3 with higher numbers overriding lower numbers. Allowed formats are json, yaml and toml.")
        .arg(
            Arg::with_name(GENERATE_CONFIG)
                .long(GENERATE_CONFIG)
                .takes_value(true)
                .possible_values(GENERATE_CONFIG_POSSIBLE)
                .value_name("format")
                .help("Prints a full configuration to stdout")
        )
        .arg(
            Arg::with_name(POLL_SECS)
                .long(POLL_SECS)
                .takes_value(true)
                .value_name("secs")
                .help("Secs to wait between polling")
        )
        .get_matches()
}

fn create_default_config() -> config::Config {
    config::Config {
            pdf: config::PDfDimension::default(),
            printer: Some(config::Printer {
                media: String::from("Custom.62x100m2"),
                orientation: String::from("landscape"),
                number_of_copies: 2,
                name: String::from("<printer name>"),
            }),
            trello: Some(config::Trello {
                app_key: String::from("<trello app key>"),
                token: String::from("<trello user token>"),
                print_label: String::from("<label to find tickets>"),
                limit_to_boards: vec![String::from("<Optional boards to limit search. Empty array to search all boards.>")],
            }),
            jira: Some(config::Jira {
                host: String::from("<jira host>"),
                user: String::from("<jira user>"),
                token: String::from("<jira user token>"),
                print_label: String::from("<label to find tickets>"),
                limit_to_projects: vec![String::from("<Optional projects to limit search. Empty array to search all projects.>")],
            })
        }
}

fn stringify_config(
    config_type: &str,
    config: &config::Config,
) -> crate::Result<String> {
    let config_text = match config_type {
        "json" => serde_json::to_string_pretty(&config)
            .with_context(|_| {
                "Could not fetch Trello Board".to_string()
            }),
        "yaml" => {
            serde_yaml::to_string(&config).with_context(|_| {
                "Could not fetch Trello Board".to_string()
            })
        }
        "toml" => toml::to_string(&config).with_context(|_| {
            "Could not fetch Trello Board".to_string()
        }),
        _ => {
            return Err(failure::err_msg(
                "Invalid configuration type",
            )
            .into())
        }
    }?;
    Ok(config_text)
}
