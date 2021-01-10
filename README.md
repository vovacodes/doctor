<div align="center">
  <h1>🩺 doctor</h1>
  <p>
    <strong>Fast and flexible low level parser for JavaDoc-style doc comments.</strong>
  </p>
  <p>
    <a href="https://crates.io/crates/doctor"><img alt="crates.io" src="https://meritbadge.herokuapp.com/doctor"/></a>
    <a href="https://docs.rs/doctor"><img alt="docs.rs" src="https://docs.rs/doctor/badge.svg"/></a>
  </p>
</div>

## Usage

```rust
use doctor::parse;
use doctor::ast::{DocComment, Description, BodyItem, BlockTag, InlineTag};

assert_eq!(
  parse(
    r#"/**
        * This is a doc comment.
        * It contains an {@inlineTag with some body} in its description.
        *
        * @blockTag1
        * @blockTag2 with body text
        * @blockTag3 with body text and {@inlineTag}
        */"#
  ),
  Ok(DocComment {
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
  })
);
```

For additional info check the [documentation](https://docs.rs/doctor).

## 🔮 Design Goals

- The crate is agnostic from the concrete set of valid tags so that more high-level parsers (JSDoc, TSDoc, etc.) can be built on top of it.
- The parser tries to allocate as little memory as possible to ensure great performance,
  so the AST format is designed to use slices of the input data as much as possible.
  
## 👯‍ Contributing

Please check [the contributing documentation](./CONTRIBUTING.md)
