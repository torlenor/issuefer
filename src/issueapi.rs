#[derive(Clone)]
pub struct Issue {
    pub number: i64,
    pub title: String,
    pub state: String,
}

pub trait IssueAPI {
    fn get_issues(&self) -> Option<Vec<Issue>>;
    fn get_closed_issues(&self) -> Option<Vec<Issue>>;
    fn create_issue(&self, title: &str) -> Option<Issue>;
    fn repo(&self) -> String;
}
