//! <div align="center">
//!   <h1>🩺 doctor</h1>
//!   <p>
//!     <strong>Fast and flexible low level parser for JavaDoc-style doc comments.</strong>
//!   </p>
//!   <p>
//!     <a href="https://crates.io/crates/doctor"><img alt="crates.io" src="https://meritbadge.herokuapp.com/doctor"/></a>
//!     <a href="https://docs.rs/doctor"><img alt="docs.rs" src="https://docs.rs/doctor/0.3.2"/></a>
//!   </p>
//! </div>
//!
//! ## Example
//!
//! ```rust
//! use doctor::parse;
//! use doctor::ast::{DocComment, Description, BodyItem, BlockTag, InlineTag};
//!
//! assert_eq!(
//!     parse(
//!         r#"/**
//!             * This is a doc comment.
//!             * It contains an {@inlineTag with some body} in its description.
//!             *
//!             * @blockTag1
//!             * @blockTag2 with body text
//!             * @blockTag3 with body text and {@inlineTag}
//!             */"#
//!     ),
//!     Ok(DocComment {
//!         description: Some(Description {
//!             body_items: vec![
//!                 BodyItem::TextSegment("This is a doc comment.\n"),
//!                 BodyItem::TextSegment("It contains an "),
//!                 BodyItem::InlineTag(InlineTag {
//!                     name: "inlineTag",
//!                     body_lines: vec!["with some body"],
//!                 }),
//!                 BodyItem::TextSegment("in its description.\n"),
//!                 BodyItem::TextSegment("\n"),
//!             ]
//!         }),
//!         block_tags: vec![
//!             BlockTag {
//!                 name: "blockTag1",
//!                 body_items: vec![]
//!             },
//!             BlockTag {
//!                 name: "blockTag2",
//!                 body_items: vec![BodyItem::TextSegment("with body text\n"),]
//!             },
//!             BlockTag {
//!                 name: "blockTag3",
//!                 body_items: vec![
//!                     BodyItem::TextSegment("with body text and "),
//!                     BodyItem::InlineTag(InlineTag {
//!                         name: "inlineTag",
//!                         body_lines: vec![]
//!                     }),
//!                     BodyItem::TextSegment("\n"),
//!                 ]
//!             },
//!         ]       
//!     })
//! );
//! ```
//! For additional info check the [documentation](https://docs.rs/doctor).
//!
//! ## 🔮 Design Goals
//!
//! - The crate is agnostic from the concrete set of valid tags so that more high-level parsers (JSDoc, TSDoc, etc.) can be built on top of it.
//! - The parser tries to allocate as little memory as possible to ensure great performance,
//!   so the AST format is designed to use slices of the input data as much as possible.
//!   
//! ## 👯‍ Contributing
//!
//! Please check [the contributing documentation](./CONTRIBUTING.md)
//!

#![doc(html_root_url = "https://docs.rs/doctor/0.3.2")]
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
