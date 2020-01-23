use crate::{
    config::Jira,
    services::{Service, Ticket},
};
use failure::ResultExt;
use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    Client, Error, RequestBuilder, Response, Url,
};
use serde::Deserialize;
use std::vec::IntoIter;

pub fn fetch_tickets(
    jira: &Jira,
    tickets: &mut Vec<Ticket>,
) -> crate::Result<()> {
    let query = build_query(jira);
    let issues = IssueListIterator::new(jira, query);
    for issue in issues {
        let issue = issue?;
        remove_label(jira, &issue.id).with_context(|_| {
            format!(
                "Could not remove tag from issue {}",
                issue.id
            )
        })?;
        tickets.push(Ticket {
            id: issue.id.clone(),
            label_id: jira.print_label.clone(),
            titel: issue.fields.summary,
            subtitel: issue.key,
            url: issue.url,
            service: Service::Jira,
        });
    }
    Ok(())
}

fn build_query(jira: &Jira) -> String {
    let mut projects = jira
        .limit_to_projects
        .iter()
        .map(|entry| format!("project = {}", entry))
        .collect::<Vec<String>>()
        .join(" OR ");
    if !projects.is_empty() {
        projects = format!("{} AND", projects);
    }
    let types = jira.limit_to_types.join(", ");
    if !types.is_empty() {
        projects = format!("issuetype in ({}) AND", projects);
    }
    format!(
        "{} {} labels = {}",
        projects, types, jira.print_label
    )
}

pub fn revert_tickets(jira: &Jira, tickets: &[Ticket]) {
    for ticket in tickets {
        if let Service::Jira = ticket.service {
            let _ = add_label(jira, &ticket.id);
        }
    }
}

fn fetch_resource(
    builder: RequestBuilder,
    jira: &Jira,
) -> Result<Response, Error> {
    builder
        .basic_auth(&jira.user, Some(&jira.token))
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .send()
        .and_then(reqwest::Response::error_for_status)
}

struct IssueListIterator<'a> {
    current_list: Option<IntoIter<Issue>>,
    start_at: usize,
    fetch_more: bool,
    jira: &'a Jira,
    query: String,
    client: Client,
}

impl<'a> IssueListIterator<'a> {
    pub fn new(jira: &'a Jira, query: String) -> Self {
        IssueListIterator {
            current_list: None,
            start_at: 0,
            fetch_more: true,
            jira,
            query,
            client: Client::new(),
        }
    }

    fn fetch_new_list(&mut self) -> Result<(), Error> {
        let list = IssueList::fetch(
            &self.client,
            self.jira,
            self.start_at,
            &self.query,
        )?;
        self.current_list = Some(list.issues.into_iter());
        self.start_at += 1;
        self.fetch_more = list.max_results == list.total;
        Ok(())
    }
}

impl Iterator for IssueListIterator<'_> {
    type Item = Result<Issue, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current_list {
                Some(ref mut list) => {
                    if let Some(v) = list.next() {
                        return Some(Ok(v));
                    } else {
                        let _ = self.current_list.take();
                    }
                }
                None => {
                    if self.fetch_more {
                        if let Err(err) = self.fetch_new_list() {
                            return Some(Err(err));
                        };
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}

#[derive(Deserialize, Debug)]
struct IssueList {
    #[serde(rename = "maxResults")]
    pub max_results: u64,
    pub total: u64,
    pub issues: Vec<Issue>,
}

impl IssueList {
    pub fn fetch(
        client: &Client,
        jira: &Jira,
        start_at: usize,
        query: &str,
    ) -> Result<Self, Error> {
        let url = Url::parse_with_params(
            &format!("https://{}/rest/api/2/search", jira.host),
            &[
                ("jql", query),
                ("startAt", &start_at.to_string()),
                ("maxResults", &50.to_string()),
                ("oldIssueView", "true"),
            ],
        )
        .expect("Unable to build url");
        let req = client.get(url);
        fetch_resource(req, jira)?.json()
    }
}

#[derive(Deserialize, Debug)]
struct Issue {
    pub id: String,
    #[serde(rename = "self")]
    pub url: String,
    pub key: String,
    pub fields: Fields,
}

#[derive(Deserialize, Debug)]
struct Fields {
    pub summary: String,
}

fn remove_label(
    jira: &Jira,
    issue_id: &str,
) -> Result<(), Error> {
    let client = Client::new();
    let req = client
        .put(&format!(
            "https://{}/rest/api/3/issue/{}?oldIssueView=true",
            jira.host, issue_id
        ))
        .body(format!(
            "{{\"update\":{{\"labels\":[{{\"remove\":\"{}\"}}]}}}}",
            jira.print_label
        ));
    let _ = fetch_resource(req, jira)?;
    Ok(())
}

fn add_label(jira: &Jira, issue_id: &str) -> Result<(), Error> {
    let client = Client::new();
    let req = client
        .put(&format!(
            "https://{}/rest/api/3/issue/{}?oldIssueView=true",
            jira.host, issue_id
        ))
        .body(format!(
            "{{\"update\":{{\"labels\":[{{\"add\":\"{}\"}}]}}}}",
            jira.print_label
        ));
    let _ = fetch_resource(req, jira)?;
    Ok(())
}
