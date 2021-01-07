#![type_length_limit = "115452926"]

pub mod ast;
mod parsers;

use nom::error::convert_error;
use nom::{Finish, Parser};

use ast::DocComment;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

/// Parses `input` into a `DocComment` struct representing the doc comment's AST.
///
/// # Examples
///
/// ```
/// use doctor::parse;
/// use doctor::ast::{DocComment, Description, DescriptionBodyItem, InlineTag};
///
/// assert_eq!(
///     parse(r#"/**
///         * This is a doc comment.
///         * It contains an {@inlineTag with some body} in its description.
///         */"#
///     ),
///     Ok(DocComment {
///         description: Some(Description {
///             body_items: vec![
///                 DescriptionBodyItem::TextSegment("This is a doc comment.\n"),
///                 DescriptionBodyItem::TextSegment("It contains an "),
///                 DescriptionBodyItem::InlineTag(InlineTag {
///                     name: "inlineTag",
///                     body_lines: vec!["with some body"],
///                 }),
///                 DescriptionBodyItem::TextSegment("in its description.\n")
///             ]
///         }),
///         block_tags: vec![]
///     }),
/// );
/// ```
pub fn parse(input: &str) -> Result<DocComment, String> {
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
    fn test_parse() {
        assert_eq!(
            parse("/** Comment */ not comment"),
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
