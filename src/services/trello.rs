use crate::{
    config::Trello,
    services::{Service, Ticket},
};
use failure::ResultExt;
use reqwest::{Client, Error, Response, Url};
use serde::Deserialize;

pub fn fetch_tickets(
    trello: &Trello,
    tickets: &mut Vec<Ticket>,
) -> crate::Result<()> {
    let mut iter_a;
    let mut iter_b;
    let boards = get_boards(&trello.app_key, &trello.token)
        .with_context(|_| {
            "Could not fetch Trello Board".to_string()
        })?;
    let boards_filter: &mut dyn Iterator<Item = &Board> =
        if trello.limit_to_boards.is_empty() {
            iter_b = boards.iter().filter(|_| true);
            &mut iter_b
        } else {
            iter_a = boards.iter().filter(move |board| {
                trello.limit_to_boards.contains(&board.name)
            });
            &mut iter_a
        };
    for board in boards_filter {
        let mut lists =
            get_lists(board, &trello.token, &trello.app_key)
                .with_context(|_| {
                    format!(
                "Could not fetch Trello Board Lists for {}",
                board.name
            )
                })?;
        for list in &mut lists {
            if let Some(cards) = list.cards.take() {
                for card in cards {
                    handle_card(card, trello, tickets)?;
                }
            }
        }
    }
    Ok(())
}

fn handle_card(
    card: Card,
    trello: &Trello,
    tickets: &mut Vec<Ticket>,
) -> crate::Result<()> {
    for label in card.labels {
        if label.name == trello.print_label {
            let name = &card.name;
            remove_label(
                &card.id,
                &label,
                &trello.token,
                &trello.app_key,
            )
            .with_context(|_| {
                format!(
                    "Could not remove Label {} from Card {}",
                    &label.name, name
                )
            })?;
            tickets.push(Ticket {
                id: card.id.clone(),
                label_id: label.id,
                titel: card.name,
                subtitel: card.id,
                url: card.url,
                service: Service::Trello,
            });
            break;
        }
    }
    Ok(())
}

pub fn revert_tickets(trello: &Trello, tickets: &[Ticket]) {
    for ticket in tickets {
        if let Service::Trello = ticket.service {
            let _ = add_label(
                &ticket.id,
                &ticket.label_id,
                &trello.token,
                &trello.app_key,
            );
        }
    }
}

fn get_resource(
    url: &str,
    params: &[(&str, &str)],
) -> Result<Response, Error> {
    let url = Url::parse_with_params(url, params)
        .expect("Unable to build url");;
    reqwest::get(url)?.error_for_status()
}

#[derive(Deserialize, Debug)]
struct Board {
    pub id: String,
    pub name: String,
}

fn get_boards(
    key: &str,
    token: &str,
) -> Result<Vec<Board>, Error> {
    let mut resp = get_resource(
        "https://api.trello.com/1/members/me/boards",
        &[("key", key), ("token", token), ("filter", "open")],
    )?;
    resp.json()
}

#[derive(Deserialize, Debug)]
struct List {
    pub cards: Option<Vec<Card>>,
}

fn get_lists(
    board: &Board,
    token: &str,
    key: &str,
) -> Result<Vec<List>, Error> {
    let mut resp = get_resource(
        &format!(
            "https://api.trello.com/1/boards/{}/lists",
            &board.id
        ),
        &[("key", key), ("token", token), ("cards", "open")],
    )?;
    resp.json()
}

#[derive(Deserialize, Debug)]
struct Card {
    pub id: String,
    pub desc: String,
    pub name: String,
    pub url: String,
    pub labels: Vec<Label>,
}

#[derive(Deserialize, Debug)]
struct Label {
    pub name: String,
    pub id: String,
}

fn remove_label(
    card_id: &str,
    label: &Label,
    token: &str,
    key: &str,
) -> Result<(), Error> {
    let url = Url::parse_with_params(
        &format!(
            "https://api.trello.com/1/cards/{}/idLabels/{}",
            card_id, label.id
        ),
        &[("token", token), ("key", key)],
    )
    .expect("Unable to build url");
    let client = Client::new();
    let resp = client.delete(url).send()?;
    let _ = resp.error_for_status()?;
    Ok(())
}

fn add_label(
    card_id: &str,
    label_id: &str,
    token: &str,
    key: &str,
) -> Result<(), Error> {
    let url = Url::parse_with_params(
        &format!(
            "https://api.trello.com/1/cards/{}/idLabels",
            card_id
        ),
        &[("token", token), ("key", key), ("value", label_id)],
    )
    .expect("Unable to build url");
    let client = Client::new();
    let resp = client.post(url).send()?;
    let _ = resp.error_for_status()?;
    Ok(())
}
