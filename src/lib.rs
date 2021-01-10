#![doc(html_root_url = "https://docs.rs/doctor/0.3.1")]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

pub mod ast;
pub mod error;
mod parsers;

use nom::error::convert_error;
use nom::Finish;

use ast::DocComment;
use error::Error;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

/// Parses `input` into a `DocComment` struct representing the doc comment's AST.
///
/// # Examples
///
/// ```
/// use doctor::parse;
/// use doctor::ast::{DocComment, Description, BodyItem, InlineTag, BlockTag};
///
/// assert_eq!(
///     parse(r#"/**
///         * This is a doc comment.
///         * It contains an {@inlineTag with some body} in its description.
///         *
///         * @blockTag1
///         * @blockTag2 with body text
///         * @blockTag3 with body text and {@inlineTag}
///         */"#
///     ),
///     Ok(DocComment {
///         description: Some(Description {
///             body_items: vec![
///                 BodyItem::TextSegment("This is a doc comment.\n"),
///                 BodyItem::TextSegment("It contains an "),
///                 BodyItem::InlineTag(InlineTag {
///                     name: "inlineTag",
///                     body_lines: vec!["with some body"],
///                 }),
///                 BodyItem::TextSegment("in its description.\n"),
///                 BodyItem::TextSegment("\n"),
///             ]
///         }),
///         block_tags: vec![
///             BlockTag {
///                 name: "blockTag1",
///                 body_items: vec![]
///             },
///             BlockTag {
///                 name: "blockTag2",
///                 body_items: vec![BodyItem::TextSegment("with body text\n"),]
///             },
///             BlockTag {
///                 name: "blockTag3",
///                 body_items: vec![
///                     BodyItem::TextSegment("with body text and "),
///                     BodyItem::InlineTag(InlineTag {
///                         name: "inlineTag",
///                         body_lines: vec![]
///                     }),
///                     BodyItem::TextSegment("\n"),
///                 ]
///             },
///         ]
///     }),
/// );
/// ```
///
/// # Errors
///
/// If `input` is not a valid doc comment, an error explaining where the parsing failed is returned.  
///
pub fn parse(input: &str) -> Result<DocComment, Error> {
    parsers::doc_comment(input)
        .finish()
        .map(|(_, doc)| doc)
        .map_err(|err| Error::ParseError(convert_error(input, err)))
}

#[cfg(doctest)]
doc_comment::doctest!("../README.md", readme);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_invalid() {
        assert_eq!(
            parse("/** Comment */ not comment"),
            Err(Error::ParseError(
                r#"0: at line 1, in Eof:
/** Comment */ not comment
              ^

1: at line 1, in doc_comment:
/** Comment */ not comment
^

"#
                .to_owned()
            ))
        )
    }
}
