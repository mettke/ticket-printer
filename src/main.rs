//! A program for printing jira or trello tickets on small cards
//!
//! # Configuration
//!
//! There is a limited configuration available to supply
//! credentials for jira or trello and to allow a rudimentary
//! personalisation of the printed cards.
//!
//! ```toml
//! [pdf]
//! height = 62.0
//! width = 100.0
//! margin = 4.0
//! title_lines = 2
//! title_seperator_margin = 4.0
//! qrcode_seperator_margin = 4.0
//! subtitle_size = 4.0
//!
//! [printer]
//! media = 'Custom.62x100m'
//! orientation = 'landscape'
//! number_of_copies = 2
//! name = 'Brother_QL_700'
//!
//! [trello]
//! app_key = '<APP KEY>'
//! token = '<USER TOKEN>'
//! # Tickets are filtered using the following label
//! # Label is removed after printing
//! print_label = '<LABEL>'
//! # Use an empty array to search all boards
//! limit_to_boards = ["Example Board"]
//!
//! [jira]
//! # Hostname only. Http is not supported
//! # Example: test.atlassian.com
//! host = '<JIRA HOSTNAME>'
//! user = '<USERNAME OR MAIL>'
//! token = '<USER TOKEN>'
//! # Tickets are filtered using the following label
//! # Label is removed after printing
//! print_label = '<LABEL>'
//! # Use an empty array to search all boards
//! limit_to_projects = ["Example Board"]
//! ```
//!
//! There is also an example configuration available in the
//! config folder. It is not necessary to define both jira and
//! trello. Simply remove the configuration part for the service
//! you don't want to use.
//!
//! The location of the configuration file is system dependent
//! and can be found here:
//! <https://docs.rs/directories/0.10.0/directories/struct.ProjectDirs.html#method.config_dir>
//!
//! # qrcode
//!
//! The qrcode contains a directlink to the trello or jira
//! ticket. It will, however, only show up if the url is small
//! enough. In the default configuration, the url size may not
//! be bigger then 23 lettes.
//!
//! # Installation
//!
//! On a system with rust installed you can install this package
//! using:
//!
//! ```sh
//! cargo install https://github.com/mettke/ticket-printer
//! ```
//!
//! Otherwise you may want to try a binary distributed with each
//! version
//!

// enable additional rustc warnings
#![warn(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    box_pointers,
    deprecated_in_future,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    indirect_structural_match,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    missing_doc_code_examples,
    non_ascii_idents,
    private_doc_tests,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    // unreachable_pub,
    unsafe_code,
    unstable_features,
    unused_extern_crates,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]
// enable additional clippy warnings
#![warn(clippy::cargo)]
#![warn(clippy::pedantic)]
#![warn(clippy::restriction)]
#![warn(clippy::nursery)]
// Allow some cargo
#![allow(clippy::multiple_crate_versions)]
// Allow some pedanctic
#![allow(
    clippy::integer_arithmetic,
    clippy::float_arithmetic,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
// Allow some restriction
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::print_stdout,
    clippy::wildcard_enum_match_arm
)]

mod config;
mod pdf;
mod services;

use crate::{
    config::Config, pdf::print_tickets, services::Ticket,
};
use exitfailure::ExitFailure;
use human_panic::setup_panic;
use pkg_version::{
    pkg_version_major, pkg_version_minor, pkg_version_patch,
};
use std::{env, process::exit, result};

type Result<T> = result::Result<T, ExitFailure>;

const MAJOR: u32 = pkg_version_major!();
const MINOR: u32 = pkg_version_minor!();
const PATCH: u32 = pkg_version_patch!();

fn main() -> Result<()> {
    setup_panic!();
    print_version();
    let config = config::get()?;
    if !config.service_available() {
        eprintln!("No Service configured. You may want to adopt the configuration file.");
        exit(1);
    }
    let mut tickets = Vec::new();
    if let Err(err) = fetch_tickets(&config, &mut tickets) {
        revert_tickets(&config, &tickets);
        return Err(err);
    }
    if let Err(err) = print_tickets(&config, &mut tickets) {
        revert_tickets(&config, &tickets);
        return Err(err);
    }
    Ok(())
}

fn print_version() {
    let mut args = env::args();
    let _ = args.next();
    match args.next().as_ref().map(|s| s.as_ref()) {
        Some("-v") | Some("--version") => {
            println!("{}.{}.{}", MAJOR, MINOR, PATCH);
            exit(0);
        }
        _ => {}
    }
}

fn fetch_tickets(
    config: &Config,
    tickets: &mut Vec<Ticket>,
) -> Result<()> {
    if let Some(ref trello) = config.trello {
        services::trello::fetch_tickets(trello, tickets)?;
    }
    if let Some(ref jira) = config.jira {
        services::jira::fetch_tickets(jira, tickets)?;
    }
    Ok(())
}

fn revert_tickets(config: &Config, tickets: &[Ticket]) {
    if let Some(ref trello) = config.trello {
        services::trello::revert_tickets(trello, tickets);
    }
    if let Some(ref jira) = config.jira {
        services::jira::revert_tickets(jira, tickets);
    }
}
