use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct OrgError {
    pub message: String,
}

impl Error for OrgError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for OrgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Org Error: {}", self.message)
    }
}
