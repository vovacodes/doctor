#![type_length_limit = "115452926"]

mod ast;
mod parsers;

use nom::error::convert_error;
use nom::{Finish, Parser};

use ast::DocComment;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

/// Parses `input` into a `DocComment` struct representing the doc comment's AST.
pub fn parse_doc_comment(input: &str) -> Result<DocComment, String> {
    parsers::doc_comment()
        .parse(input)
        .finish()
        .map(|(_, doc)| doc)
        .map_err(|err| convert_error(input, err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_doc_comment() {
        assert_eq!(
            parse_doc_comment("/** Comment */ not comment"),
            Err(r#"0: at line 1, in Eof:
/** Comment */ not comment
              ^

1: at line 1, in doc_comment:
/** Comment */ not comment
^

"#
            .to_owned())
        )
    }
}
