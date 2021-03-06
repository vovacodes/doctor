use nom::branch::alt;
use nom::bytes::complete::{escaped, is_not, tag};
use nom::character::complete::{
    alphanumeric1, char, line_ending, multispace0, one_of, space0, space1,
};
use nom::character::streaming::alpha1;
use nom::combinator::{all_consuming, not, opt, recognize, verify};
use nom::error::{context, make_error, ErrorKind, VerboseError};
use nom::multi::{fold_many1, many0, separated_list1};
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::{IResult, Parser};

use crate::ast::{BlockTag, BodyItem, Description, DocComment, InlineTag};

/// Eats the doc comment start sequence.
fn comment_start(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    context(
        "comment_start",
        tuple((tag("/**"), space0, opt(line_ending))),
    )
    .map(|_| ())
    .parse(i)
}

/// Eats the doc comment end sequence.
fn comment_end(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    context("comment_end", tuple((multispace0, tag("*/"))))
        .map(|_| ())
        .parse(i)
}

/// Parses a single comment line leading, i.e. ` * `.
fn line_leading(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    context(
        "line_leading",
        recognize(tuple((space0, not(tag("*/")), tag("*"), space0))),
    )
    .parse(i)
}

/// Parses an inline or block tag name.
fn tag_name(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    context(
        "tag_name",
        preceded(
            tag("@"),
            recognize(pair(alpha1, many0(alt((alphanumeric1, tag("_")))))),
        ),
    )
    .parse(i)
}

/// Returns an error if the parsed output of the provided parser is empty.
fn non_empty<'a>(
    mut parser: impl Parser<&'a str, &'a str, VerboseError<&'a str>>,
) -> impl Parser<&'a str, &'a str, VerboseError<&'a str>> {
    move |i: &'a str| {
        let result = parser.parse(i)?;
        if result.1.is_empty() {
            Err(nom::Err::Error(make_error(i, ErrorKind::NonEmpty)))
        } else {
            Ok(result)
        }
    }
}

/// Parses a single line of an inline tag's body.
fn inline_tag_body_line(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    context(
        "inline_tag_body_line",
        alt((
            line_ending,
            recognize(tuple((
                non_empty(escaped(is_not("\\\r\n{}"), '\\', one_of("{}"))),
                opt(line_ending),
            ))),
        )),
    )
    .parse(i)
}

/// Parses an inline tag's body.
/// It might contain multiple lines of text.
fn inline_tag_body(i: &str) -> IResult<&str, Vec<&str>, VerboseError<&str>> {
    context(
        "inline_tag_body",
        separated_list1(line_leading, inline_tag_body_line),
    )
    .parse(i)
}

fn inline_tag(i: &str) -> IResult<&str, InlineTag<'_>, VerboseError<&str>> {
    context(
        "inline_tag",
        delimited(
            char('{'),
            tuple((tag_name, opt(preceded(opt(space1), inline_tag_body)))),
            preceded(opt(line_leading), char('}')),
        ),
    )
    .map(|(name, maybe_body_lines)| InlineTag {
        name,
        body_lines: maybe_body_lines.unwrap_or_else(Vec::new),
    })
    .parse(i)
}

/// Parses an single text segment of a description's or block tag's body.
/// A segment is usually terminated by either an inline tag or a line ending.
fn body_text_segment(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    context(
        "body_text_segment",
        alt((
            line_ending,
            recognize(tuple((
                verify(
                    take_until_either(&[
                        Token::Escapable("{"),
                        Token::Escapable("}"),
                        Token::Escapable("@"),
                        Token::NonEscapable("\r"),
                        Token::NonEscapable("\n"),
                        Token::NonEscapable("*/"),
                    ]),
                    // The segment has to be non-empty and not whitespace-only.
                    |s: &str| {
                        !s.is_empty() && s.chars().any(|ch| !ch.is_whitespace() && ch != '\t')
                    },
                ),
                opt(line_ending),
            ))),
        )),
    )
    .parse(i)
}

/// Parses body of a description or a block tag.
fn body(i: &str) -> IResult<&str, Vec<BodyItem<'_>>, VerboseError<&str>> {
    #[derive(Debug)]
    enum ParsedEntities<'a> {
        BodyItem(BodyItem<'a>),
        Ignored,
    }

    verify(
        fold_many1(
            alt((
                line_leading.map(|_| ParsedEntities::Ignored),
                space1.map(|_| ParsedEntities::Ignored),
                inline_tag
                    .map(BodyItem::InlineTag)
                    .map(ParsedEntities::BodyItem),
                body_text_segment
                    .map(BodyItem::TextSegment)
                    .map(ParsedEntities::BodyItem),
            )),
            vec![],
            |mut items, item| {
                if let ParsedEntities::BodyItem(item) = item {
                    items.push(item)
                }
                items
            },
        ),
        |body_items: &Vec<BodyItem>| {
            // Don't consider empty or whitespace-only lines a body.
            body_items.iter().any(|item| match item {
                BodyItem::TextSegment(s) => !is_empty_or_multispace(s),
                BodyItem::InlineTag(_) => true,
            })
        },
    )
    .parse(i)
}

/// Parses a description section of a doc comment.
fn description(i: &str) -> IResult<&str, Description<'_>, VerboseError<&str>> {
    context("description", body)
        .map(|body_items| Description { body_items })
        .parse(i)
}

/// Parses a single block tag.
fn block_tag(i: &str) -> IResult<&str, BlockTag<'_>, VerboseError<&str>> {
    context("block_tag", tuple((tag_name, space0, opt(body))))
        .map(|(name, _, maybe_body_items)| BlockTag {
            name,
            body_items: maybe_body_items.unwrap_or_else(Vec::new),
        })
        .parse(i)
}

/// Parses an entire doc comment.
pub fn doc_comment(i: &str) -> IResult<&str, DocComment<'_>, VerboseError<&str>> {
    context(
        "doc_comment",
        all_consuming(tuple((
            comment_start,
            opt(line_leading),
            opt(description),
            many0(delimited(opt(line_leading), block_tag, opt(line_ending))),
            comment_end,
        ))),
    )
    .map(|(_, _, description, block_tags, _)| DocComment {
        description,
        block_tags,
    })
    .parse(i)
}

#[derive(Debug)]
enum Token<'a> {
    Escapable(&'a str),
    NonEscapable(&'a str),
}

fn take_until_either<'a>(
    tokens: &'a [Token<'a>],
) -> impl Parser<&'a str, &'a str, VerboseError<&'a str>> {
    move |input: &'a str| {
        let mut escaping = false;
        let chars = input.char_indices();
        for (i, ch) in chars {
            let next_escaping = ch == '\\' && !escaping;
            if next_escaping {
                escaping = next_escaping;
                continue;
            }

            for token in tokens {
                let found = match token {
                    Token::Escapable(t) => !escaping && input[i..].starts_with(t),
                    Token::NonEscapable(t) => input[i..].starts_with(t),
                };
                if found {
                    let (parsed, rest) = input.split_at(i);
                    return Ok((rest, parsed));
                };
            }

            escaping = next_escaping;
        }

        // Returning an empty &str as the "rest" causes a runtime panic in code that works with this "rest".
        // I didn't fully understand why that happens but returning an empty subslice of `input` fixes the problem.
        // I suppose the issue is somehow related to some internal state that `input` holds.
        Ok((&input[input.len()..], input))
    }
}

fn is_empty_or_multispace(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    for ch in s.chars() {
        if !ch.is_whitespace() && ch != '\t' && ch != '\n' && ch != '\r' {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use nom::error::{ErrorKind, VerboseErrorKind};
    use nom::Err as NomErr;

    use super::*;

    /// Utility function that allows to inspect the parser result without consuming it.
    // fn tap<'a, O>(
    //     mut parser: impl Parser<&'a str, O, VerboseError<&'a str>>,
    //     f: impl Fn(&IResult<&'a str, O, VerboseError<&'a str>>),
    // ) -> impl Parser<&'a str, O, VerboseError<&'a str>> {
    //     move |i: &'a str| {
    //         let result = parser.parse(i);
    //         f(&result);
    //         result
    //     }
    // }

    #[test]
    fn test_comment_start() {
        assert_eq!(comment_start("/**"), Ok(("", ())));
        assert_eq!(comment_start("/**   \n"), Ok(("", ())));
        assert_eq!(
            comment_start("/** the rest of the line"),
            Ok(("the rest of the line", ()))
        );
        assert_eq!(
            comment_start("/*"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("/*", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    ("/*", VerboseErrorKind::Context("comment_start"))
                ]
            }))
        );
    }

    #[test]
    fn test_comment_end() {
        assert_eq!(comment_end("*/"), Ok(("", ())));
        assert_eq!(comment_end("\t */"), Ok(("", ())));
        assert_eq!(comment_end("\n */"), Ok(("", ())));
        assert_eq!(
            comment_end("*/this is not comment anymore"),
            Ok(("this is not comment anymore", ()))
        );
        assert_eq!(
            comment_end("*"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("*", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    ("*", VerboseErrorKind::Context("comment_end"))
                ]
            }))
        );
    }

    #[test]
    fn test_line_leading() {
        assert_eq!(line_leading("*"), Ok(("", "*")));
        assert_eq!(line_leading(" * "), Ok(("", " * ")));
        assert_eq!(
            line_leading(" * text after the separator"),
            Ok(("text after the separator", " * "))
        );

        assert_eq!(
            line_leading(" */ "),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("*/ ", VerboseErrorKind::Nom(ErrorKind::Not)),
                    (" */ ", VerboseErrorKind::Context("line_leading"))
                ]
            }))
        );
        assert_eq!(
            line_leading(" \n * "),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("\n * ", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    (" \n * ", VerboseErrorKind::Context("line_leading"))
                ]
            }))
        );
        assert_eq!(
            line_leading("text"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("text", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    ("text", VerboseErrorKind::Context("line_leading"))
                ]
            }))
        );
    }

    #[test]
    fn test_tag_name() {
        assert_eq!(tag_name("@my_tag"), Ok(("", "my_tag")));
        assert_eq!(tag_name("@myTag1"), Ok(("", "myTag1")));
        assert_eq!(tag_name("@myTag1 the rest"), Ok((" the rest", "myTag1")));
        assert_eq!(
            tag_name("myTag1"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("myTag1", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    ("myTag1", VerboseErrorKind::Context("tag_name"))
                ]
            }))
        );
        assert_eq!(
            tag_name("@1myTag"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("1myTag", VerboseErrorKind::Nom(ErrorKind::Alpha)),
                    ("@1myTag", VerboseErrorKind::Context("tag_name"))
                ]
            }))
        );
        assert_eq!(
            tag_name("@_myTag"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("_myTag", VerboseErrorKind::Nom(ErrorKind::Alpha)),
                    ("@_myTag", VerboseErrorKind::Context("tag_name"))
                ]
            }))
        );
    }

    #[test]
    fn test_inline_tag_body_line() {
        assert_eq!(inline_tag_body_line("\n"), Ok(("", "\n")));
        assert_eq!(inline_tag_body_line("Hello"), Ok(("", "Hello")));
        assert_eq!(inline_tag_body_line("Hello\n"), Ok(("", "Hello\n")));
        assert_eq!(inline_tag_body_line("Hello}"), Ok(("}", "Hello")));
        assert_eq!(
            inline_tag_body_line("Hello { world"),
            Ok(("{ world", "Hello "))
        );
        assert_eq!(inline_tag_body_line("He\\}llo}"), Ok(("}", "He\\}llo")));
        assert_eq!(
            inline_tag_body_line("Hello \\{\\} world"),
            Ok(("", "Hello \\{\\} world"))
        );

        assert_eq!(
            inline_tag_body_line(""),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("", VerboseErrorKind::Nom(ErrorKind::NonEmpty)),
                    ("", VerboseErrorKind::Nom(ErrorKind::Alt)),
                    ("", VerboseErrorKind::Context("inline_tag_body_line"))
                ]
            }))
        );
        assert_eq!(
            inline_tag_body_line("Hello \\ world"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    (" world", VerboseErrorKind::Nom(ErrorKind::OneOf)),
                    ("Hello \\ world", VerboseErrorKind::Nom(ErrorKind::Alt)),
                    (
                        "Hello \\ world",
                        VerboseErrorKind::Context("inline_tag_body_line")
                    )
                ]
            }))
        );
    }

    #[test]
    fn test_inline_tag_body() {
        let input = r#"Hello
        * world.
        * \{\}
        *
        * Second paragraph.
        * }"#;
        assert_eq!(
            inline_tag_body(input),
            Ok((
                "        * }",
                vec![
                    "Hello\n",
                    "world.\n",
                    "\\{\\}\n",
                    "\n",
                    "Second paragraph.\n"
                ]
            ))
        );
    }

    #[test]
    fn test_inline_tag() {
        assert_eq!(
            inline_tag("{@tag}"),
            Ok((
                "",
                InlineTag {
                    name: "tag",
                    body_lines: vec![]
                }
            ))
        );
        assert_eq!(
            inline_tag("{@tag body text}"),
            Ok((
                "",
                InlineTag {
                    name: "tag",
                    body_lines: vec!["body text"]
                }
            ))
        );
        assert_eq!(
            inline_tag("{@tag - body text}"),
            Ok((
                "",
                InlineTag {
                    name: "tag",
                    body_lines: vec!["- body text"]
                }
            ))
        );
        assert_eq!(
            inline_tag("{@tag \\{\\}}"),
            Ok((
                "",
                InlineTag {
                    name: "tag",
                    body_lines: vec!["\\{\\}"]
                }
            ))
        );
        assert_eq!(
            inline_tag("{@tag @body}"),
            Ok((
                "",
                InlineTag {
                    name: "tag",
                    body_lines: vec!["@body"]
                }
            ))
        );
        assert_eq!(
            inline_tag("{@tag\n * line 1\n * line 2}"),
            Ok((
                "",
                InlineTag {
                    name: "tag",
                    body_lines: vec!["\n", "line 1\n", "line 2"]
                }
            ))
        );
    }

    #[test]
    fn test_body_text_segment() {
        assert_eq!(body_text_segment("\n"), Ok(("", "\n")));
        assert_eq!(
            body_text_segment("Hello {@ world\n"),
            Ok(("{@ world\n", "Hello "))
        );
        assert_eq!(
            body_text_segment("Hello */ world"),
            Ok(("*/ world", "Hello "))
        );
        assert_eq!(
            body_text_segment("Hello \\{@ world\n"),
            Ok(("@ world\n", "Hello \\{"))
        );
        assert_eq!(
            body_text_segment("Hello \\{\\@ world\n"),
            Ok(("", "Hello \\{\\@ world\n"))
        );
        assert_eq!(
            body_text_segment("Hello \\\\{@ world\n"),
            Ok(("{@ world\n", "Hello \\\\"))
        );
        assert_eq!(
            body_text_segment("Hello \\\\\\{ world\n"),
            Ok(("", "Hello \\\\\\{ world\n"))
        );
        assert_eq!(
            body_text_segment("Hello world\r\n"),
            Ok(("", "Hello world\r\n"))
        );
        assert_eq!(
            body_text_segment(""),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("", VerboseErrorKind::Nom(ErrorKind::Verify)),
                    ("", VerboseErrorKind::Nom(ErrorKind::Alt)),
                    ("", VerboseErrorKind::Context("body_text_segment"))
                ]
            }))
        );
        assert_eq!(
            body_text_segment("   \t "),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("   \t ", VerboseErrorKind::Nom(ErrorKind::Verify)),
                    ("   \t ", VerboseErrorKind::Nom(ErrorKind::Alt)),
                    ("   \t ", VerboseErrorKind::Context("body_text_segment"))
                ]
            }))
        );
        assert_eq!(
            body_text_segment("{"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("{", VerboseErrorKind::Nom(ErrorKind::Verify)),
                    ("{", VerboseErrorKind::Nom(ErrorKind::Alt)),
                    ("{", VerboseErrorKind::Context("body_text_segment"))
                ]
            }))
        );
        assert_eq!(
            body_text_segment("@"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("@", VerboseErrorKind::Nom(ErrorKind::Verify)),
                    ("@", VerboseErrorKind::Nom(ErrorKind::Alt)),
                    ("@", VerboseErrorKind::Context("body_text_segment"))
                ]
            }))
        );
    }

    #[test]
    fn test_description() {
        assert_eq!(
            description(
                r#"This is the description section
            * that contains
            * multiple lines
            *
            * and paragraphs.
            * @blockTag"#
            ),
            Ok((
                "@blockTag",
                Description {
                    body_items: vec![
                        BodyItem::TextSegment("This is the description section\n"),
                        BodyItem::TextSegment("that contains\n"),
                        BodyItem::TextSegment("multiple lines\n"),
                        BodyItem::TextSegment("\n"),
                        BodyItem::TextSegment("and paragraphs.\n"),
                    ]
                }
            ))
        );
        assert_eq!(
            description(
                r#"This is the description section
            * that contains both text segments and {@inlineTag}.
            * @blockTag"#
            ),
            Ok((
                "@blockTag",
                Description {
                    body_items: vec![
                        BodyItem::TextSegment("This is the description section\n"),
                        BodyItem::TextSegment("that contains both text segments and "),
                        BodyItem::InlineTag(InlineTag {
                            name: "inlineTag",
                            body_lines: vec![]
                        }),
                        BodyItem::TextSegment(".\n"),
                    ]
                }
            ))
        );
        assert_eq!(
            description(
                r#"This is the description section
            * that contains multi-line {@inlineTag
            * tag body
            * }
            * @blockTag"#
            ),
            Ok((
                "@blockTag",
                Description {
                    body_items: vec![
                        BodyItem::TextSegment("This is the description section\n"),
                        BodyItem::TextSegment("that contains multi-line "),
                        BodyItem::InlineTag(InlineTag {
                            name: "inlineTag",
                            body_lines: vec!["\n", "tag body\n"]
                        }),
                        BodyItem::TextSegment("\n"),
                    ]
                }
            ))
        );
        assert_eq!(
            description("{@inlineTag with body}    \n"),
            Ok((
                "",
                Description {
                    body_items: vec![
                        BodyItem::InlineTag(InlineTag {
                            name: "inlineTag",
                            body_lines: vec!["with body"]
                        }),
                        BodyItem::TextSegment("\n"),
                    ]
                }
            ))
        );
    }

    #[test]
    fn test_block_tag() {
        assert_eq!(
            block_tag("@blockTag "),
            Ok((
                "",
                BlockTag {
                    name: "blockTag",
                    body_items: vec![]
                }
            ))
        );
        assert_eq!(
            block_tag("@blockTag*/"),
            Ok((
                "*/",
                BlockTag {
                    name: "blockTag",
                    body_items: vec![]
                }
            ))
        );
        assert_eq!(
            block_tag("@blockTag with body */"),
            Ok((
                "*/",
                BlockTag {
                    name: "blockTag",
                    body_items: vec![BodyItem::TextSegment("with body ")]
                }
            ))
        );
        assert_eq!(
            block_tag(r#"@blockTag with body @anotherBlockTag"#),
            Ok((
                "@anotherBlockTag",
                BlockTag {
                    name: "blockTag",
                    body_items: vec![BodyItem::TextSegment("with body ")]
                }
            ))
        );
        assert_eq!(
            block_tag(
                r#"@blockTag with body
                * @anotherBlockTag"#
            ),
            Ok((
                "@anotherBlockTag",
                BlockTag {
                    name: "blockTag",
                    body_items: vec![BodyItem::TextSegment("with body\n")]
                }
            ))
        );
        assert_eq!(
            block_tag("@blockTag {@inlineTag}"),
            Ok((
                "",
                BlockTag {
                    name: "blockTag",
                    body_items: vec![BodyItem::InlineTag(InlineTag {
                        name: "inlineTag",
                        body_lines: vec![]
                    })]
                }
            ))
        );
    }

    #[test]
    fn test_comment_empty() {
        assert_eq!(
            doc_comment("/** */"),
            Ok((
                "",
                DocComment {
                    description: None,
                    block_tags: vec![],
                }
            ))
        );
    }

    #[test]
    fn test_comment_one_line_description() {
        assert_eq!(
            doc_comment("/** One-line description. */"),
            Ok((
                "",
                DocComment {
                    description: Some(Description {
                        body_items: vec![BodyItem::TextSegment("One-line description. ")]
                    }),
                    block_tags: vec![],
                }
            ))
        );
        assert_eq!(
            doc_comment("/** One-line description containing {@inlineTag} */"),
            Ok((
                "",
                DocComment {
                    description: Some(Description {
                        body_items: vec![
                            BodyItem::TextSegment("One-line description containing "),
                            BodyItem::InlineTag(InlineTag {
                                name: "inlineTag",
                                body_lines: vec![]
                            })
                        ]
                    }),
                    block_tags: vec![],
                }
            ))
        );
        assert_eq!(
            doc_comment(
                "/** One-line description containing {@inlineTag} and some text after it. */"
            ),
            Ok((
                "",
                DocComment {
                    description: Some(Description {
                        body_items: vec![
                            BodyItem::TextSegment("One-line description containing "),
                            BodyItem::InlineTag(InlineTag {
                                name: "inlineTag",
                                body_lines: vec![]
                            }),
                            BodyItem::TextSegment("and some text after it. "),
                        ]
                    }),
                    block_tags: vec![],
                }
            ))
        );
        assert_eq!(
            doc_comment("/** One-line description containing {@inlineTag with body} */"),
            Ok((
                "",
                DocComment {
                    description: Some(Description {
                        body_items: vec![
                            BodyItem::TextSegment("One-line description containing "),
                            BodyItem::InlineTag(InlineTag {
                                name: "inlineTag",
                                body_lines: vec!["with body"]
                            }),
                        ]
                    }),
                    block_tags: vec![],
                }
            ))
        );
    }

    #[test]
    fn test_comment_multi_line() {
        assert_eq!(
            doc_comment(
                r#"/**
                * This is a description-only comment.
                * The description contains an {@inlineTag} though.
                */"#
            ),
            Ok((
                "",
                DocComment {
                    description: Some(Description {
                        body_items: vec![
                            BodyItem::TextSegment("This is a description-only comment.\n"),
                            BodyItem::TextSegment("The description contains an "),
                            BodyItem::InlineTag(InlineTag {
                                name: "inlineTag",
                                body_lines: vec![],
                            }),
                            BodyItem::TextSegment("though.\n")
                        ]
                    }),
                    block_tags: vec![]
                }
            ))
        );
    }

    #[test]
    fn test_comment_all_elements() {
        assert_eq!(
            doc_comment(
                r#"/**
                * This is a doc comment.
                * It contains an {@inlineTag with some body} in its description.
                *
                * @blockTag1
                * @blockTag2 with body text
                * @blockTag3 with body text and {@inlineTag}
                */"#
            ),
            Ok((
                "",
                DocComment {
                    description: Some(Description {
                        body_items: vec![
                            BodyItem::TextSegment("This is a doc comment.\n"),
                            BodyItem::TextSegment("It contains an "),
                            BodyItem::InlineTag(InlineTag {
                                name: "inlineTag",
                                body_lines: vec!["with some body"],
                            }),
                            BodyItem::TextSegment("in its description.\n"),
                            BodyItem::TextSegment("\n"),
                        ]
                    }),
                    block_tags: vec![
                        BlockTag {
                            name: "blockTag1",
                            body_items: vec![]
                        },
                        BlockTag {
                            name: "blockTag2",
                            body_items: vec![BodyItem::TextSegment("with body text\n"),]
                        },
                        BlockTag {
                            name: "blockTag3",
                            body_items: vec![
                                BodyItem::TextSegment("with body text and "),
                                BodyItem::InlineTag(InlineTag {
                                    name: "inlineTag",
                                    body_lines: vec![]
                                }),
                                BodyItem::TextSegment("\n"),
                            ]
                        },
                    ]
                }
            ))
        )
    }
}
