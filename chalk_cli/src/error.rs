use std::io;

#[derive(Debug)]
pub enum ActionError {
    Connect(Box<ureq::Error>),
    Read(io::Error),
    Parse(serde_json::Error),
}

impl From<ureq::Error> for ActionError {
    fn from(from: ureq::Error) -> Self {
        Self::Connect(Box::new(from))
    }
}

impl From<io::Error> for ActionError {
    fn from(from: io::Error) -> Self {
        Self::Read(from)
    }
}

impl From<serde_json::Error> for ActionError {
    fn from(from: serde_json::Error) -> Self {
        Self::Parse(from)
    }
}
