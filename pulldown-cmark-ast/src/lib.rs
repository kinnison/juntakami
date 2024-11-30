//! An AST model for [pulldown_cmark] supporting "reserialisation"
//!

pub(crate) mod ast;
mod parse;

#[cfg(any(test, feature = "generate"))]
mod generate;

#[doc(inline)]
pub use ast::*;

#[cfg(feature = "generate")]
pub mod fold;

pub use pulldown_cmark::Options as ParseOptions;

#[cfg(feature = "generate")]
pub use pulldown_cmark_to_cmark::Options as RenderOptions;

#[cfg(test)]
mod test {
    use fold::MarkdownFold;
    use insta::{assert_debug_snapshot, assert_snapshot};
    use pulldown_cmark::{Event, Parser};

    use super::*;

    const EVERYTHING: &str = r###"
# Purpose

This constant covers everything that pulldown-cmark can generate.  If this parses and
can be reconstituted then we're good. [^1]

[^1]: Footnote one

## Attributes { #name .foo .bar baz=cake wibble .glug .boo }

![Stuff](image.png)

It's also important to support [reflinks] and [normal links](somewhere)

[reflinks]: https://cheese.com

> Here's a block quote
> Which is multiple lines

```
foo bar
```

    indented code block
    which has more than one line

<strong>Eww!</strong>

<div class="cake">
Markdown [link](ignored) and **bold** not applied
</div>

# Some mathematics

You can $inline$ it.

Or you can display it: $$ x = 2 
y = 4 $$

Some Formatting
===============

This block starts with an underlined header.  
We also have a hard break above
and a soft break after.

We can do _emphasised_ text, **strong** text, and ~struck-through~ text.

---

Let's play with tasks next

- [ ] Traditional incomplete task

  With some extra text
- [x] Traditional complete task
- [d] Task to be dropped
- [.] Partially complete task

1. A numbered list
2. [x] done

# Definition lists

First Term
: This is the definition of the first term.

Second Term
: This is one definition of the second term.
: This is another definition of the second term.

# Trailing metadata

+++
foo = "bar"
wibble = [1, 2, 3]
+++

The above is still metadata, as is the below

---
yaml: metadata
...

# Table stuff

| Syntax      | Description | Test Text     |
| :---        |    :----:   |          ---: |
| Header      | Title       | Here's this   |
| Paragraph   | Text        | And more      |

| Syntax      | Description | Test Text     |
| ---        |    ----   |          --- |
| Header      | Title       | Here's this   |
| Paragraph   | Text        | And more      |

# We had some funky list issues

- [D] should vanish
  even though it's multiline
- [.] should become dash
  - Doing more stuff
- [x] Should become F


"###;

    // The options we want for EVERYTHING
    fn opts() -> pulldown_cmark::Options {
        use pulldown_cmark::Options;
        Options::empty()
            | Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_HEADING_ATTRIBUTES
            | Options::ENABLE_DEFINITION_LIST
            | Options::ENABLE_MATH
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_DEFINITION_LIST
            | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
            | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
    }

    fn noisy_parser(
        text: &str,
        options: pulldown_cmark::Options,
    ) -> impl Iterator<Item = Event<'_>> {
        Parser::new_ext(text, options).map(|e| {
            eprintln!("Event: {e:?}");
            e
        })
    }

    fn render_opts() -> pulldown_cmark_to_cmark::Options<'static> {
        use pulldown_cmark_to_cmark::Options;
        Options {
            list_token: '-',
            increment_ordered_list_bullets: true,
            emphasis_token: '_',
            strong_token: "**",
            ordered_list_token: '.',
            ..Default::default()
        }
    }

    #[test]
    fn roundtrip() {
        let doc = Document::from_events(noisy_parser(EVERYTHING, opts()));
        assert_debug_snapshot!(doc);
        struct NullFolder;
        impl MarkdownFold for NullFolder {}
        let rendered = NullFolder.fold_document(doc).render(render_opts());
        assert_snapshot!(rendered);
    }
}
