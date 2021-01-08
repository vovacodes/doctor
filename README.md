<div align="center">
  <h1>🩺 doctor</h1>
  <p>
    <strong>Fast and flexible low level parser for JavaDoc-style doc comments</strong>
  </p>
  <p>
    <a href="https://crates.io/crates/doctor"><img alt="crates.io" src="https://meritbadge.herokuapp.com/doctor"/></a>
    <a href="https://docs.rs/doctor"><img alt="docs.rs" src="https://docs.rs/doctor/badge.svg"/></a>
  </p>
</div>

## About

This crate seeks to be a low-level parser for
[Javadoc-style](https://www.oracle.com/technical-resources/articles/java/javadoc-tool.html#format) doc comment formats,
e.g. JavaDoc, JSDoc, TSDoc, etc.

## Example

```rust
use doctor::parse;
use doctor::ast::{DocComment, Description, DescriptionBodyItem, InlineTag};

assert_eq!(
    parse(r#"/**
        * This is a doc comment.
        * It contains an {@inlineTag with some body} in its description.
        */"#
    ),
    Ok(DocComment {
        description: Some(Description {
            body_items: vec![
                DescriptionBodyItem::TextSegment("This is a doc comment.\n"),
                DescriptionBodyItem::TextSegment("It contains an "),
                DescriptionBodyItem::InlineTag(InlineTag {
                    name: "inlineTag",
                    body_lines: vec!["with some body"],
                }),
                DescriptionBodyItem::TextSegment("in its description.\n")
            ]
        }),
        block_tags: vec![]
    }),
);
```

## 🔮 Design Goals

- The crate is agnostic from the concrete set of valid tags so that more high-level parsers (JSDoc, TSDoc, etc.) can be built on top of it.
- The parser tries to allocate as little memory as possible to ensure great performance,
  so the AST format is designed to use slices of the input data as much as possible.
  
## 👯‍ Contributing

Please check [the contributing documentation](./CONTRIBUTING.md)