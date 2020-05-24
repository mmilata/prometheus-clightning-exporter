use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct AnError {
    msg: String,
}

impl fmt::Display for AnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", self.msg)
    }
}

impl Error for AnError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl AnError {
    pub fn new(msg: &str) -> Self {
        AnError {
            msg: msg.to_string(),
        }
    }
}

type Result<T> = std::result::Result<T, AnError>;
