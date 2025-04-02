

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Normal,
    Error,
}

pub struct Toast {
    pub title: String,
    pub body: String,
    pub status: Status,
}

impl Toast {
    pub fn new<S: AsRef<str>>(title: S, body: String, status: Status) -> Self {
        Self {
            title: title.as_ref().to_string(),
            body,
            status
        }
    }
}

