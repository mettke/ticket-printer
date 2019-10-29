use crate::config::{
    self, Global, Jira, PDfDimension, Printer, Trello,
};
use clap::{
    app_from_crate, crate_authors, crate_description, crate_name,
    crate_version, Arg,
};
use failure::ResultExt;
use std::process::exit;

const GENERATE_CONFIG: &str = "generate-example-config";
const PRINT_CONFIG: &str = "print-current-config";
const CONFIG_POSSIBLE: &[&str; 3] = &["json", "yaml", "toml"];
const POLL_SECS: &str = "poll";
const POLL_SECS_ENV: &str = "POLL_SECS";

const PDF_HEIGHT: &str = "pdf-height";
const PDF_HEIGHT_ENV: &str = "PDF_HEIGHT";
const PDF_WIDTH: &str = "pdf-width";
const PDF_WIDTH_ENV: &str = "PDF_WIDTH";
const PDF_MARGIN: &str = "pdf-margin";
const PDF_MARGIN_ENV: &str = "PDF_MARGIN";
const PDF_TITLE_LINES: &str = "pdf-title-lines";
const PDF_TITLE_LINES_ENV: &str = "PDF_TITLE_LINES";
const PDF_TITLE_SEPERATOR_MARGIN: &str =
    "pdf-title-seperator-margin";
const PDF_TITLE_SEPERATOR_MARGIN_ENV: &str =
    "PDF_TITLE_SEPERATOR_MARGIN";
const PDF_QRCODE_SEPERATOR_MARGIN: &str =
    "pdf-qrcode-seperator-margin";
const PDF_QRCODE_SEPERATOR_MARGIN_ENV: &str =
    "PDF_QRCODE_SEPERATOR_MARGIN";
const PDF_SUBTITLE_SIZE: &str = "pdf-subtitle-size";
const PDF_SUBTITLE_SIZE_ENV: &str = "PDF_SUBTITLE_SIZE";
const PDF_ARGUMENTS: &[&str] = &[
    PDF_HEIGHT,
    PDF_WIDTH,
    PDF_MARGIN,
    PDF_TITLE_LINES,
    PDF_TITLE_SEPERATOR_MARGIN,
    PDF_QRCODE_SEPERATOR_MARGIN,
    PDF_SUBTITLE_SIZE,
];

const PRINTER_MEDIA: &str = "printer-media";
const PRINTER_MEDIA_ENV: &str = "PRINTER_MEDIA";
const PRINTER_ORIENTATION: &str = "printer-orientation";
const PRINTER_ORIENTATION_ENV: &str = "PRINTER_ORIENTATION";
const PRINTER_NUMBER_OF_COPIES: &str = "printer-number-of-copies";
const PRINTER_NUMBER_OF_COPIES_ENV: &str =
    "PRINTER_NUMBER_OF_COPIES";
const PRINTER_NAME: &str = "printer-name";
const PRINTER_NAME_ENV: &str = "PRINTER_NAME";
const PRINTER_ARGUMENTS: &[&str] = &[
    PRINTER_MEDIA,
    PRINTER_ORIENTATION,
    PRINTER_NUMBER_OF_COPIES,
    PRINTER_NAME,
];

const TRELLO_APP_KEY: &str = "trello-app-key";
const TRELLO_APP_KEY_ENV: &str = "TRELLO_APP_KEY";
const TRELLO_TOKEN: &str = "trello-token";
const TRELLO_TOKEN_ENV: &str = "TRELLO_TOKEN";
const TRELLO_PRINT_LABEL: &str = "trello-print-label";
const TRELLO_PRINT_LABEL_ENV: &str = "TRELLO_PRINT_LABEL";
const TRELLO_LIMIT_TO_BOARDS: &str = "trello-limit-to-boards";
const TRELLO_LIMIT_TO_BOARDS_ENV: &str = "TRELLO_LIMIT_TO_BOARDS";
const TRELLO_ARGUMENTS: &[&str] =
    &[TRELLO_APP_KEY, TRELLO_TOKEN, TRELLO_PRINT_LABEL];

const JIRA_HOST: &str = "jira-host";
const JIRA_HOST_ENV: &str = "JIRA_HOST";
const JIRA_USER: &str = "jira-user";
const JIRA_USER_ENV: &str = "JIRA_USER";
const JIRA_TOKEN: &str = "jira-token";
const JIRA_TOKEN_ENV: &str = "JIRA_TOKEN";
const JIRA_PRINT_LABEL: &str = "jira-print-label";
const JIRA_PRINT_LABEL_ENV: &str = "JIRA_PRINT_LABEL";
const JIRA_LIMIT_TO_PROJECTS: &str = "jira-limit-to-projects";
const JIRA_LIMIT_TO_PROJECTS_ENV: &str = "JIRA_LIMIT_TO_PROJECTS";
const JIRA_ARGUMENTS: &[&str] =
    &[JIRA_HOST, JIRA_USER, JIRA_TOKEN, JIRA_PRINT_LABEL];

#[derive(Debug)]
pub struct Arguments {
    pdf: Option<PDfDimension>,
    printer: Option<Printer>,
    trello: Option<Trello>,
    jira: Option<Jira>,
    poll: Option<u64>,
    print: Option<String>,
}

impl Arguments {
    pub fn merge_config(
        self,
        config: &mut config::Config,
    ) -> crate::Result<()> {
        if let Some(pdf) = self.pdf {
            config.pdf = pdf;
        }
        if let Some(printer) = self.printer {
            config.printer = Some(printer);
        }
        if let Some(trello) = self.trello {
            config.trello = Some(trello);
        }
        if let Some(jira) = self.jira {
            config.jira = Some(jira);
        }
        if let Some(poll) = self.poll {
            config.global = Some(Global { poll: Some(poll) });
        }
        if let Some(format) = self.print.as_ref() {
            let config_text = stringify_config(format, config)?;
            println!("{}", config_text);
            exit(0);
        }
        Ok(())
    }
}

impl Default for Arguments {
    fn default() -> Self {
        Self {
            pdf: None,
            printer: None,
            trello: None,
            jira: None,
            poll: None,
            print: None,
        }
    }
}

pub fn handle() -> crate::Result<Arguments> {
    let mut arguments = Arguments::default();
    let matches = setup();
    if let Some(config_type) = matches.value_of(GENERATE_CONFIG) {
        let config = create_default_config();
        let config_text = stringify_config(config_type, &config)?;
        println!("{}", config_text);
        exit(0);
    }
    arguments.print =
        matches.value_of(PRINT_CONFIG).map(|s| s.into());
    arguments.poll =
        matches.value_of(POLL_SECS).and_then(|s| s.parse().ok());

    if matches.is_present(PDF_HEIGHT) {
        arguments.pdf = Some(PDfDimension {
            height: matches
                .value_of(PDF_HEIGHT)
                .expect("CLAP REQUIRES")
                .parse::<f32>()
                .with_context(|_| {
                    format!("{} must be numeric", PDF_HEIGHT)
                })?,
            width: matches
                .value_of(PDF_WIDTH)
                .expect("CLAP REQUIRES")
                .parse::<f32>()
                .with_context(|_| {
                    format!("{} must be numeric", PDF_WIDTH)
                })?,
            margin: matches
                .value_of(PDF_MARGIN)
                .expect("CLAP REQUIRES")
                .parse::<f32>()
                .with_context(|_| {
                    format!("{} must be numeric", PDF_MARGIN)
                })?,
            title_lines: matches
                .value_of(PDF_TITLE_LINES)
                .expect("CLAP REQUIRES")
                .parse::<u32>()
                .with_context(|_| {
                    format!("{} must be numeric", PDF_TITLE_LINES)
                })?,
            title_seperator_margin: matches
                .value_of(PDF_TITLE_SEPERATOR_MARGIN)
                .expect("CLAP REQUIRES")
                .parse::<f32>()
                .with_context(|_| {
                    format!(
                        "{} must be numeric",
                        PDF_TITLE_SEPERATOR_MARGIN
                    )
                })?,
            qrcode_seperator_margin: matches
                .value_of(PDF_QRCODE_SEPERATOR_MARGIN)
                .expect("CLAP REQUIRES")
                .parse::<f32>()
                .with_context(|_| {
                    format!(
                        "{} must be numeric",
                        PDF_QRCODE_SEPERATOR_MARGIN
                    )
                })?,
            subtitle_size: matches
                .value_of(PDF_SUBTITLE_SIZE)
                .expect("CLAP REQUIRES")
                .parse::<f32>()
                .with_context(|_| {
                    format!(
                        "{} must be numeric",
                        PDF_SUBTITLE_SIZE
                    )
                })?,
        })
    }
    if matches.is_present(PRINTER_MEDIA) {
        arguments.printer = Some(Printer {
            media: matches
                .value_of(PRINTER_MEDIA)
                .expect("CLAP REQUIRES")
                .into(),
            orientation: matches
                .value_of(PRINTER_ORIENTATION)
                .expect("CLAP REQUIRES")
                .into(),
            number_of_copies: matches
                .value_of(PRINTER_NUMBER_OF_COPIES)
                .expect("CLAP REQUIRES")
                .parse::<u16>()
                .with_context(|_| {
                    format!(
                        "{} must be numeric",
                        PRINTER_NUMBER_OF_COPIES
                    )
                })?,
            name: matches
                .value_of(PRINTER_NAME)
                .expect("CLAP REQUIRES")
                .into(),
        })
    }
    if matches.is_present(TRELLO_APP_KEY) {
        arguments.trello = Some(Trello {
            app_key: matches
                .value_of(TRELLO_APP_KEY)
                .expect("CLAP REQUIRES")
                .into(),
            token: matches
                .value_of(TRELLO_TOKEN)
                .expect("CLAP REQUIRES")
                .into(),
            print_label: matches
                .value_of(TRELLO_PRINT_LABEL)
                .expect("CLAP REQUIRES")
                .into(),
            limit_to_boards: matches
                .values_of(TRELLO_LIMIT_TO_BOARDS)
                .map_or_else(Vec::new, |i| {
                    i.map(|s| s.into()).collect()
                }),
        })
    }
    if matches.is_present(JIRA_HOST) {
        arguments.jira = Some(Jira {
            host: matches
                .value_of(JIRA_HOST)
                .expect("CLAP REQUIRES")
                .into(),
            user: matches
                .value_of(JIRA_USER)
                .expect("CLAP REQUIRES")
                .into(),
            token: matches
                .value_of(JIRA_TOKEN)
                .expect("CLAP REQUIRES")
                .into(),
            print_label: matches
                .value_of(JIRA_PRINT_LABEL)
                .expect("CLAP REQUIRES")
                .into(),
            limit_to_projects: matches
                .values_of(JIRA_LIMIT_TO_PROJECTS)
                .map_or_else(Vec::new, |i| {
                    i.map(|s| s.into()).collect()
                }),
        })
    }
    Ok(arguments)
}

fn setup() -> clap::ArgMatches<'static> {
    app_from_crate!()
        .after_help("You can setup your configuration file in the following places:

    1. /etc/ticket_printer/ticket_printer.{ext}
    2. ~/.config/ticket_printer/ticket_printer.{ext}
    3. ticket_printer.{ext}
    4. Environment Variables
    5. Command Line Parameters

Configuration is merged from 1 to 5 with higher numbers overriding lower numbers. Allowed formats are json, yaml and toml.")
        .arg(
            Arg::with_name(GENERATE_CONFIG)
                .long(GENERATE_CONFIG)
                .takes_value(true)
                .possible_values(CONFIG_POSSIBLE)
                .value_name("format")
                .help("Prints a full configuration to stdout")
        )
        .arg(
            Arg::with_name(PRINT_CONFIG)
                .long(PRINT_CONFIG)
                .takes_value(true)
                .possible_values(CONFIG_POSSIBLE)
                .value_name("format")
                .help("Prints the current configuration to stdout")
        )
        .arg(
            Arg::with_name(POLL_SECS)
                .long(POLL_SECS)
                .takes_value(true)
                .value_name("secs")
                .env(POLL_SECS_ENV)
                .help("Secs to wait between polling\n[conf: global.poll]")
        )
        .arg(
            Arg::with_name(PDF_HEIGHT)
                .long(PDF_HEIGHT)
                .takes_value(true)
                .value_name("height")
                .env(PDF_HEIGHT_ENV)
                .help("Height of the output pdf\n[conf: pdf.height]")
                .requires_all(PDF_ARGUMENTS)
        )
        .arg(
            Arg::with_name(PDF_WIDTH)
                .long(PDF_WIDTH)
                .takes_value(true)
                .value_name("width")
                .env(PDF_WIDTH_ENV)
                .help("Width of the output pdf\n[conf: pdf.width]")
                .requires_all(PDF_ARGUMENTS)
        )
        .arg(
            Arg::with_name(PDF_MARGIN)
                .long(PDF_MARGIN)
                .takes_value(true)
                .value_name("margin")
                .env(PDF_MARGIN_ENV)
                .help("Border margin of the output pdf\n[conf: pdf.margin]")
                .requires_all(PDF_ARGUMENTS)
        )
        .arg(
            Arg::with_name(PDF_TITLE_LINES)
                .long(PDF_TITLE_LINES)
                .takes_value(true)
                .value_name("count")
                .env(PDF_TITLE_LINES_ENV)
                .help("Number of lines for the title\n[conf: pdf.title_lines]")
                .requires_all(PDF_ARGUMENTS)
        )
        .arg(
            Arg::with_name(PDF_TITLE_SEPERATOR_MARGIN)
                .long(PDF_TITLE_SEPERATOR_MARGIN)
                .takes_value(true)
                .value_name("margin")
                .env(PDF_TITLE_SEPERATOR_MARGIN_ENV)
                .help("Margin between title and lower content\n[conf: pdf.title_seperator_margin]")
                .requires_all(PDF_ARGUMENTS)
        )
        .arg(
            Arg::with_name(PDF_QRCODE_SEPERATOR_MARGIN)
                .long(PDF_QRCODE_SEPERATOR_MARGIN)
                .takes_value(true)
                .value_name("margin")
                .env(PDF_QRCODE_SEPERATOR_MARGIN_ENV)
                .help("Margin between qrcode and subtitle\n[conf: pdf.qrcode_seperator_margin]")
                .requires_all(PDF_ARGUMENTS)
        )
        .arg(
            Arg::with_name(PDF_SUBTITLE_SIZE)
                .long(PDF_SUBTITLE_SIZE)
                .takes_value(true)
                .value_name("size")
                .env(PDF_SUBTITLE_SIZE_ENV)
                .help("Font size of the subtitle\n[conf: pdf.subtitle_size]")
                .requires_all(PDF_ARGUMENTS)
        )
        .arg(
            Arg::with_name(PRINTER_MEDIA)
                .long(PRINTER_MEDIA)
                .takes_value(true)
                .value_name("paper type")
                .env(PRINTER_MEDIA_ENV)
                .help("Type of the output paper\n[conf: printer.media]")
                .requires_all(PRINTER_ARGUMENTS)
        )
        .arg(
            Arg::with_name(PRINTER_ORIENTATION)
                .long(PRINTER_ORIENTATION)
                .takes_value(true)
                .value_name("orientation")
                .env(PRINTER_ORIENTATION_ENV)
                .help("paper orientation\n[conf: printer.orientation]")
                .requires_all(PRINTER_ARGUMENTS)
        )
        .arg(
            Arg::with_name(PRINTER_NUMBER_OF_COPIES)
                .long(PRINTER_NUMBER_OF_COPIES)
                .takes_value(true)
                .value_name("count")
                .env(PRINTER_NUMBER_OF_COPIES_ENV)
                .help("Number of copies\n[conf: printer.number_of_copies]")
                .requires_all(PRINTER_ARGUMENTS)
        )
        .arg(
            Arg::with_name(PRINTER_NAME)
                .long(PRINTER_NAME)
                .takes_value(true)
                .value_name("printer name")
                .env(PRINTER_NAME_ENV)
                .help("Name of the printer\n[conf: printer.name]")
                .requires_all(PRINTER_ARGUMENTS)
        )
        .arg(
            Arg::with_name(TRELLO_APP_KEY)
                .long(TRELLO_APP_KEY)
                .takes_value(true)
                .value_name("app key")
                .env(TRELLO_APP_KEY_ENV)
                .help("trello app key\n[conf: trello.app_key]")
                .requires_all(TRELLO_ARGUMENTS)
        )
        .arg(
            Arg::with_name(TRELLO_TOKEN)
                .long(TRELLO_TOKEN)
                .takes_value(true)
                .value_name("token")
                .env(TRELLO_TOKEN_ENV)
                .help("trello access token\n[conf: trello.token]")
                .requires_all(TRELLO_ARGUMENTS)
        )
        .arg(
            Arg::with_name(TRELLO_PRINT_LABEL)
                .long(TRELLO_PRINT_LABEL)
                .takes_value(true)
                .value_name("label")
                .env(TRELLO_PRINT_LABEL_ENV)
                .help("label to find cards\n[conf: trello.print_label]")
                .requires_all(TRELLO_ARGUMENTS)
        )
        .arg(
            Arg::with_name(TRELLO_LIMIT_TO_BOARDS)
                .long(TRELLO_LIMIT_TO_BOARDS)
                .takes_value(true)
                .value_name("board")
                .env(TRELLO_LIMIT_TO_BOARDS_ENV)
                .help("limit search to board\n[conf: trello.limit_to_boards]")
                .requires_all(TRELLO_ARGUMENTS)
                .number_of_values(1)
                .multiple(true)
        )
        .arg(
            Arg::with_name(JIRA_HOST)
                .long(JIRA_HOST)
                .takes_value(true)
                .value_name("hostname")
                .env(JIRA_HOST_ENV)
                .help("jira server hostname\n[conf: jira.host]")
                .requires_all(JIRA_ARGUMENTS)
        )
        .arg(
            Arg::with_name(JIRA_USER)
                .long(JIRA_USER)
                .takes_value(true)
                .value_name("username")
                .env(JIRA_USER_ENV)
                .help("jira username\n[conf: jira.user]")
                .requires_all(JIRA_ARGUMENTS)
        )
        .arg(
            Arg::with_name(JIRA_TOKEN)
                .long(JIRA_TOKEN)
                .takes_value(true)
                .value_name("token")
                .env(JIRA_TOKEN_ENV)
                .help("jira access token\n[conf: jira.token]")
                .requires_all(JIRA_ARGUMENTS)
        )
        .arg(
            Arg::with_name(JIRA_PRINT_LABEL)
                .long(JIRA_PRINT_LABEL)
                .takes_value(true)
                .value_name("label")
                .env(JIRA_PRINT_LABEL_ENV)
                .help("label to find tickets\n[conf: jira.print_label]")
                .requires_all(JIRA_ARGUMENTS)
        )
        .arg(
            Arg::with_name(JIRA_LIMIT_TO_PROJECTS)
                .long(JIRA_LIMIT_TO_PROJECTS)
                .takes_value(true)
                .value_name("project")
                .env(JIRA_LIMIT_TO_PROJECTS_ENV)
                .help("limit search to projects\n[conf: jira.limit_to_projects]")
                .requires_all(JIRA_ARGUMENTS)
                .number_of_values(1)
                .multiple(true)
        )
        .get_matches()
}

fn create_default_config() -> config::Config {
    config::Config {
            pdf: config::PDfDimension::default(),
            printer: Some(Printer {
                media: String::from("Custom.62x100m2"),
                orientation: String::from("landscape"),
                number_of_copies: 2,
                name: String::from("<printer name>"),
            }),
            trello: Some(Trello {
                app_key: String::from("<trello app key>"),
                token: String::from("<trello user token>"),
                print_label: String::from("<label to find tickets>"),
                limit_to_boards: vec![String::from("<Optional boards to limit search. Empty array to search all boards.>")],
            }),
            jira: Some(Jira {
                host: String::from("<jira host>"),
                user: String::from("<jira user>"),
                token: String::from("<jira user token>"),
                print_label: String::from("<label to find tickets>"),
                limit_to_projects: vec![String::from("<Optional projects to limit search. Empty array to search all projects.>")],
            }),
            global: None
        }
}

fn stringify_config(
    config_type: &str,
    config: &config::Config,
) -> crate::Result<String> {
    let config_text = match config_type {
        "json" => serde_json::to_string_pretty(&config)
            .with_context(|_| {
                "Could not deserialize config".to_string()
            }),
        "yaml" => {
            serde_yaml::to_string(&config).with_context(|_| {
                "Could not deserialize config".to_string()
            })
        }
        "toml" => toml::to_string(&config).with_context(|_| {
            "Could not deserialize config".to_string()
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
