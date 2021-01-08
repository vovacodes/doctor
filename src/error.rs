use std::fmt::{Display, Formatter, Result};

#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    ParseError(String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::ParseError(msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_error<T: std::error::Error>() {}

    #[test]
    fn test_implement_error() {
        assert_error::<Error>()
    }
}
