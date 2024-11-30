//! Generators for turning the AST back into markdown text

use pulldown_cmark::{Event, Tag, TagEnd};
use pulldown_cmark_to_cmark::Options;

use crate::ast::*;

impl Document {
    pub fn render(&self, mut options: Options) -> String {
        let mut events = Vec::new();
        self.push_events(&mut events);
        options.code_block_token_count =
            pulldown_cmark_to_cmark::calculate_code_block_token_count(&events)
                .unwrap_or(pulldown_cmark_to_cmark::DEFAULT_CODE_BLOCK_TOKEN_COUNT);
        let mut ret = String::new();
        pulldown_cmark_to_cmark::cmark_with_options(events.into_iter(), &mut ret, options).unwrap();
        ret
    }

    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        self.blocks.iter().for_each(|b| b.push_events(events))
    }
}

impl Block {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        match self {
            Block::Metadata(metadata_block) => metadata_block.push_events(events),
            Block::Heading(heading) => heading.push_events(events),
            Block::Paragraph(paragraph) => paragraph.push_events(events),
            Block::BlockQuote(block_quote) => block_quote.push_events(events),
            Block::CodeBlock(code_block) => code_block.push_events(events),
            Block::HtmlBlock(html_block) => html_block.push_events(events),
            Block::FootnoteDefinition(footnote_definition) => {
                footnote_definition.push_events(events)
            }
            Block::List(list) => list.push_events(events),
            Block::DefinitionList(definition_list) => definition_list.push_events(events),
            Block::Table(table) => table.push_events(events),
            Block::Rule => events.push(Event::Rule),
        }
    }
}

impl Inline {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        match self {
            Inline::Image(image) => image.push_events(events),
            Inline::Link(link) => link.push_events(events),
            Inline::Text(s) => events.push(Event::Text(s.clone())),
            Inline::Html(s) => events.push(Event::Html(s.clone())),
            Inline::InlineMath(s) => events.push(Event::InlineMath(s.clone())),
            Inline::DisplayMath(s) => events.push(Event::DisplayMath(s.clone())),
            Inline::Code(s) => events.push(Event::Code(s.clone())),
            Inline::FootnoteReference(s) => events.push(Event::FootnoteReference(s.clone())),
            Inline::Emphasis(vec) => {
                events.push(Event::Start(Tag::Emphasis));
                vec.iter().for_each(|i| i.push_events(events));
                events.push(Event::End(TagEnd::Emphasis));
            }
            Inline::Strong(vec) => {
                events.push(Event::Start(Tag::Strong));
                vec.iter().for_each(|i| i.push_events(events));
                events.push(Event::End(TagEnd::Strong));
            }
            Inline::Strikethrough(vec) => {
                events.push(Event::Start(Tag::Strikethrough));
                vec.iter().for_each(|i| i.push_events(events));
                events.push(Event::End(TagEnd::Strikethrough));
            }
            Inline::SoftBreak => events.push(Event::SoftBreak),
            Inline::HardBreak => events.push(Event::HardBreak),
            Inline::TasklistMarker(b) => events.push(Event::TaskListMarker(*b)),
            Inline::InlineBlock(b) => b.push_events(events),
        }
    }
}

impl MetadataBlock {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::MetadataBlock(self.kind)));
        events.push(Event::Text(self.content.clone()));
        events.push(Event::End(TagEnd::MetadataBlock(self.kind)));
    }
}

impl Heading {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::Heading {
            level: self.level,
            id: self.id.clone(),
            classes: self.classes.clone(),
            attrs: self.attrs.clone(),
        }));
        self.body.iter().for_each(|i| i.push_events(events));
        events.push(Event::End(TagEnd::Heading(self.level)))
    }
}

impl Paragraph {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::Paragraph));
        self.body.iter().for_each(|i| i.push_events(events));
        events.push(Event::End(TagEnd::Paragraph));
    }
}

impl BlockQuote {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::BlockQuote(self.kind)));
        self.body.iter().for_each(|b| b.push_events(events));
        events.push(Event::End(TagEnd::BlockQuote(self.kind)));
    }
}

impl CodeBlock {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::CodeBlock(self.kind.clone())));
        self.body.iter().for_each(|i| i.push_events(events));
        events.push(Event::End(TagEnd::CodeBlock));
    }
}

impl HtmlBlock {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::HtmlBlock));
        self.body
            .iter()
            .for_each(|t| events.push(Event::Html(t.clone())));
        events.push(Event::End(TagEnd::HtmlBlock));
    }
}

impl FootnoteDefinition {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::FootnoteDefinition(self.label.clone())));
        self.body.iter().for_each(|b| b.push_events(events));
        events.push(Event::End(TagEnd::FootnoteDefinition));
    }
}

impl List {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::List(self.start)));
        self.items.iter().for_each(|li| li.push_events(events));
        events.push(Event::End(Tag::List(self.start).into()));
    }
}

impl ListItem {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::Item));
        match self {
            ListItem::Inline(vec) => vec.iter().for_each(|i| i.push_events(events)),
            ListItem::Block(vec) => vec.iter().for_each(|b| b.push_events(events)),
        }
        events.push(Event::End(TagEnd::Item));
    }
}

impl DefinitionList {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::DefinitionList));
        self.items.iter().for_each(|i| i.push_events(events));
        events.push(Event::End(TagEnd::DefinitionList));
    }
}

impl DefinitionItem {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::DefinitionListTitle));
        self.title.iter().for_each(|i| i.push_events(events));
        events.push(Event::End(TagEnd::DefinitionListTitle));
        for def in &self.definitions {
            events.push(Event::Start(Tag::DefinitionListDefinition));
            def.body.iter().for_each(|i| i.push_events(events));
            events.push(Event::End(TagEnd::DefinitionListDefinition));
        }
    }
}

impl Table {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::Table(self.alignments.clone())));
        self.header.push_events(events);
        self.rows.iter().for_each(|r| r.push_events(events));
        events.push(Event::End(TagEnd::Table));
    }
}

impl TableHead {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        if !self.cells.is_empty() {
            events.push(Event::Start(Tag::TableHead));
            self.cells.iter().for_each(|c| c.push_events(events));
            events.push(Event::End(TagEnd::TableHead));
        }
    }
}

impl TableRow {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        if !self.cells.is_empty() {
            events.push(Event::Start(Tag::TableRow));
            self.cells.iter().for_each(|c| c.push_events(events));
            events.push(Event::End(TagEnd::TableRow));
        }
    }
}

impl TableCell {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::TableCell));
        self.body.iter().for_each(|i| i.push_events(events));
        events.push(Event::End(TagEnd::TableCell));
    }
}

impl Image {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::Image {
            link_type: self.link_type,
            dest_url: self.dest_url.clone(),
            title: self.title.clone(),
            id: self.id.clone(),
        }));
        self.body.iter().for_each(|i| i.push_events(events));
        events.push(Event::End(TagEnd::Image));
    }
}

impl Link {
    fn push_events(&self, events: &mut Vec<Event<'static>>) {
        events.push(Event::Start(Tag::Link {
            link_type: self.link_type,
            dest_url: self.dest_url.clone(),
            title: self.title.clone(),
            id: self.id.clone(),
        }));
        self.body.iter().for_each(|i| i.push_events(events));
        events.push(Event::End(TagEnd::Link));
    }
}
