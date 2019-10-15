pub mod jira;
pub mod trello;

#[derive(Debug, Clone, Copy)]
pub enum Service {
    Trello,
    Jira,
}

impl Into<&'static str> for Service {
    fn into(self) -> &'static str {
        match self {
            Self::Trello => "Trello",
            Self::Jira => "Jira",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ticket {
    pub id: String,
    pub label_id: String,
    pub titel: String,
    pub subtitel: String,
    pub url: String,
    pub service: Service,
}
