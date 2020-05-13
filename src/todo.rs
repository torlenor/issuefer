use std::fmt;

#[derive(Clone)]
pub struct Todo {
    pub file_path: String,
    pub line_number: usize,
    pub title: String,
    pub issue_number: u16,
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.issue_number == 0 {
            write!(
                f,
                "{}:{}: TODO: {}",
                self.file_path,
                self.line_number + 1,
                self.title
            )
        } else {
            write!(
                f,
                "{}:{}: TODO (#{}): {}",
                self.file_path,
                self.line_number + 1,
                self.issue_number,
                self.title
            )
        }
    }
}
