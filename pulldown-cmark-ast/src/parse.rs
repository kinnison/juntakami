use std::iter::Peekable;

use pulldown_cmark::{Alignment, CowStr, Event, MetadataBlockKind, Tag, TagEnd};

use crate::ast::*;

impl Document {
    pub fn from_events<'a>(events: impl IntoIterator<Item = Event<'a>>) -> Self {
        let mut events = events.into_iter().peekable();
        let mut blocks = Vec::new();
        while events.peek().is_some() {
            let block = Block::from_events(&mut events);
            blocks.push(block);
        }
        Self { blocks }
    }

    pub fn parse(input: &str, options: pulldown_cmark::Options) -> Self {
        let parser = pulldown_cmark::Parser::new_ext(input, options);
        Self::from_events(parser)
    }
}

impl Block {
    fn from_events<'a>(events: &mut Peekable<impl Iterator<Item = Event<'a>>>) -> Self {
        // We're only ever called for blocks, so there will always be something here
        match events.next().unwrap() {
            Event::Start(tag) => {
                // Definitely some kind of block
                match tag {
                    heading @ Tag::Heading { .. } => {
                        Self::Heading(Heading::from_events(heading, events))
                    }
                    Tag::Paragraph => Self::Paragraph(Paragraph::from_events(events)),
                    blockquote @ Tag::BlockQuote(_) => {
                        Self::BlockQuote(BlockQuote::from_events(events, blockquote))
                    }
                    codeblock @ Tag::CodeBlock(_) => {
                        Self::CodeBlock(CodeBlock::from_events(events, codeblock))
                    }
                    Tag::HtmlBlock => Self::HtmlBlock(HtmlBlock::from_events(events)),
                    footnote @ Tag::FootnoteDefinition(_) => {
                        Self::FootnoteDefinition(FootnoteDefinition::from_events(events, footnote))
                    }
                    Tag::List(start) => Self::List(List::from_events(events, start)),
                    Tag::DefinitionList => {
                        Self::DefinitionList(DefinitionList::from_events(events))
                    }
                    Tag::MetadataBlock(kind) => {
                        Self::Metadata(MetadataBlock::from_events(events, kind))
                    }
                    Tag::Table(alignments) => Self::Table(Table::from_events(events, alignments)),

                    Tag::Item
                    | Tag::DefinitionListTitle
                    | Tag::DefinitionListDefinition
                    | Tag::TableHead
                    | Tag::TableRow
                    | Tag::TableCell
                    | Tag::Emphasis
                    | Tag::Strong
                    | Tag::Strikethrough
                    | Tag::Link { .. }
                    | Tag::Image { .. } => {
                        unreachable!("The tag {tag:?} should not be reachable from Block context")
                    }
                }
            }

            Event::Rule => Self::Rule,

            Event::End(_)
            | Event::Text(_)
            | Event::Code(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::SoftBreak
            | Event::HardBreak
            | Event::TaskListMarker(_) => {
                unreachable!("Hit an event which should not be reachable from Block context")
            }
        }
    }

    fn many_from_events<'a>(
        events: &mut Peekable<impl Iterator<Item = Event<'a>>>,
        end: TagEnd,
    ) -> Vec<Self> {
        let mut ret = Vec::new();
        loop {
            if events.peek() == Some(&Event::End(end)) {
                events.next();
                break ret;
            }
            ret.push(Self::from_events(events));
        }
    }
}

impl Heading {
    fn from_events<'a>(
        tag: Tag<'a>,
        events: &mut Peekable<impl Iterator<Item = Event<'a>>>,
    ) -> Self {
        let Tag::Heading {
            level,
            id,
            classes,
            attrs,
        } = tag.clone()
        else {
            unreachable!()
        };
        let end = TagEnd::from(tag);

        let body = Inline::from_events(events, end);

        Self {
            level,
            id: id.map(CowStr::into_static),
            classes: classes.into_iter().map(CowStr::into_static).collect(),
            attrs: attrs
                .into_iter()
                .map(|(k, v)| (k.into_static(), v.map(CowStr::into_static)))
                .collect(),
            body,
        }
    }
}

impl Inline {
    fn from_events<'a>(
        events: &mut Peekable<impl Iterator<Item = Event<'a>>>,
        end: TagEnd,
    ) -> Vec<Self> {
        let mut ret = Vec::new();
        loop {
            match events.next().unwrap() {
                Event::End(t) if t == end => {
                    break ret;
                }

                Event::Text(text) => ret.push(Inline::Text(text.into_static())),
                Event::SoftBreak => ret.push(Inline::SoftBreak),
                Event::HardBreak => ret.push(Inline::HardBreak),
                Event::Start(tag) => match tag {
                    img @ Tag::Image { .. } => {
                        ret.push(Self::Image(Image::from_events(events, img)));
                    }

                    link @ Tag::Link { .. } => {
                        ret.push(Self::Link(Link::from_events(events, link)));
                    }

                    Tag::Emphasis => ret.push(Self::Emphasis(Inline::from_events(
                        events,
                        TagEnd::Emphasis,
                    ))),
                    Tag::Strong => {
                        ret.push(Self::Strong(Inline::from_events(events, TagEnd::Strong)))
                    }
                    Tag::Strikethrough => ret.push(Self::Strikethrough(Inline::from_events(
                        events,
                        TagEnd::Strikethrough,
                    ))),

                    tag => {
                        panic!("Unable to process Start({tag:?}) for inline");
                    }
                },

                Event::InlineHtml(h) => ret.push(Self::Html(h.into_static())),
                Event::InlineMath(m) => ret.push(Self::InlineMath(m.into_static())),
                Event::DisplayMath(m) => ret.push(Self::DisplayMath(m.into_static())),
                Event::Code(c) => ret.push(Self::Code(c.into_static())),
                Event::FootnoteReference(f) => ret.push(Self::FootnoteReference(f.into_static())),
                Event::TaskListMarker(b) => ret.push(Self::TasklistMarker(b)),

                e => {
                    panic!("Unable to process event {e:?} for an inline");
                }
            }
        }
    }
}

impl Paragraph {
    fn from_events<'a>(events: &mut Peekable<impl Iterator<Item = Event<'a>>>) -> Self {
        // We run until we end a paragraph
        Self {
            body: Inline::from_events(events, TagEnd::Paragraph),
        }
    }
}

impl Image {
    fn from_events<'a>(
        events: &mut Peekable<impl Iterator<Item = Event<'a>>>,
        img: Tag<'a>,
    ) -> Self {
        let Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        } = img
        else {
            unreachable!()
        };
        let body = Inline::from_events(events, TagEnd::Image);
        Self {
            link_type,
            dest_url: dest_url.into_static(),
            title: title.into_static(),
            id: id.into_static(),
            body,
        }
    }
}

impl Link {
    fn from_events<'a>(
        events: &mut Peekable<impl Iterator<Item = Event<'a>>>,
        link: Tag<'a>,
    ) -> Self {
        let Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        } = link
        else {
            unreachable!()
        };
        let body = Inline::from_events(events, TagEnd::Link);
        Self {
            link_type,
            dest_url: dest_url.into_static(),
            title: title.into_static(),
            id: id.into_static(),
            body,
        }
    }
}

impl BlockQuote {
    fn from_events<'a>(
        events: &mut Peekable<impl Iterator<Item = Event<'a>>>,
        blockquote: Tag<'a>,
    ) -> Self {
        let Tag::BlockQuote(kind) = blockquote else {
            unreachable!()
        };

        Self {
            kind,
            body: Block::many_from_events(events, TagEnd::BlockQuote(kind)),
        }
    }
}

impl CodeBlock {
    fn from_events<'a>(
        events: &mut Peekable<impl Iterator<Item = Event<'a>>>,
        codeblock: Tag<'a>,
    ) -> Self {
        let Tag::CodeBlock(kind) = codeblock else {
            unreachable!()
        };

        Self {
            kind: kind.into_static(),
            body: Inline::from_events(events, TagEnd::CodeBlock),
        }
    }
}

impl HtmlBlock {
    fn from_events<'a>(events: &mut Peekable<impl Iterator<Item = Event<'a>>>) -> Self {
        let mut body = Vec::new();
        loop {
            match events.next().unwrap() {
                Event::Html(s) => body.push(s.into_static()),
                Event::End(TagEnd::HtmlBlock) => break,
                _ => unreachable!(),
            }
        }
        Self { body }
    }
}

impl FootnoteDefinition {
    fn from_events<'a>(
        events: &mut Peekable<impl Iterator<Item = Event<'a>>>,
        footnote: Tag<'a>,
    ) -> Self {
        let Tag::FootnoteDefinition(label) = footnote else {
            unreachable!()
        };

        Self {
            label: label.into_static(),
            body: Block::many_from_events(events, TagEnd::FootnoteDefinition),
        }
    }
}

impl List {
    fn from_events<'a>(
        events: &mut Peekable<impl Iterator<Item = Event<'a>>>,
        start: Option<u64>,
    ) -> Self {
        let mut items = Vec::new();
        let end = TagEnd::from(Tag::List(start));
        loop {
            match events.next().unwrap() {
                Event::Start(Tag::Item) => items.push(ListItem {
                    body: Inline::from_events(events, TagEnd::Item),
                }),
                Event::End(e) if e == end => break,
                _ => unreachable!(),
            }
        }

        Self { start, items }
    }
}

impl DefinitionList {
    fn from_events<'a>(events: &mut Peekable<impl Iterator<Item = Event<'a>>>) -> Self {
        let mut items = Vec::new();

        let empty_item = DefinitionItem {
            title: vec![],
            definitions: vec![],
        };

        let mut item = empty_item.clone();

        loop {
            match events.next().unwrap() {
                Event::Start(Tag::DefinitionListTitle) => {
                    if !item.title.is_empty() {
                        items.push(std::mem::replace(&mut item, empty_item.clone()));
                    }
                    item.title = Inline::from_events(events, TagEnd::DefinitionListTitle);
                }
                Event::Start(Tag::DefinitionListDefinition) => {
                    item.definitions.push(Inline::from_events(
                        events,
                        TagEnd::DefinitionListDefinition,
                    ));
                }

                Event::End(TagEnd::DefinitionList) => break,

                _ => unreachable!(),
            }
        }

        if !item.title.is_empty() {
            items.push(item);
        }

        Self { items }
    }
}

impl MetadataBlock {
    fn from_events<'a>(
        events: &mut Peekable<impl Iterator<Item = Event<'a>>>,
        kind: MetadataBlockKind,
    ) -> Self {
        let Event::Text(content) = events.next().unwrap() else {
            unreachable!()
        };

        let Event::End(TagEnd::MetadataBlock(endkind)) = events.next().unwrap() else {
            unreachable!()
        };

        assert_eq!(kind, endkind);

        Self {
            kind,
            content: content.into_static(),
        }
    }
}

impl Table {
    fn from_events<'a>(
        events: &mut Peekable<impl Iterator<Item = Event<'a>>>,
        alignments: Vec<Alignment>,
    ) -> Table {
        let mut ret = Table {
            alignments,
            header: TableHead { cells: Vec::new() },
            rows: Vec::new(),
        };

        loop {
            match events.next().unwrap() {
                Event::Start(Tag::TableHead) => {
                    ret.header.cells = TableCell::from_events(events, TagEnd::TableHead)
                }
                Event::Start(Tag::TableRow) => ret.rows.push(TableRow {
                    cells: TableCell::from_events(events, TagEnd::TableRow),
                }),
                Event::End(TagEnd::Table) => break,
                _ => unreachable!(),
            }
        }

        ret
    }
}

impl TableCell {
    fn from_events<'a>(
        events: &mut Peekable<impl Iterator<Item = Event<'a>>>,
        end: TagEnd,
    ) -> Vec<Self> {
        let mut ret = Vec::new();

        loop {
            match events.next().unwrap() {
                Event::End(e) if e == end => break,
                Event::Start(Tag::TableCell) => ret.push(TableCell {
                    body: Inline::from_events(events, TagEnd::TableCell),
                }),
                _ => unreachable!(),
            }
        }

        ret
    }
}
