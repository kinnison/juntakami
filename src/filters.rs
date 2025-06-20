//! Filters for markdown trees

use std::cmp::Ordering;

use pulldown_cmark_ast::{
    fold::{self, fold_list, fold_list_item, MarkdownFold},
    Block, BlockQuote, CowStr, Document, FootnoteDefinition, HeadingLevel, Inline, List, ListItem,
};

pub struct KeepDrop {
    stack: Vec<KdMode>,
}

struct KdMode {
    keeping: bool,
    cur_keeping: bool,
    cur_level: HeadingLevel,
}

impl KdMode {
    fn new(keeping: bool) -> Self {
        KdMode {
            keeping,
            cur_keeping: keeping,
            cur_level: HeadingLevel::H1,
        }
    }
}

impl KeepDrop {
    pub fn new(keeping: bool) -> KeepDrop {
        KeepDrop {
            stack: vec![KdMode::new(keeping)],
        }
    }

    fn top(&mut self) -> &mut KdMode {
        self.stack.last_mut().unwrap()
    }

    fn fold_blocklist(&mut self, blocks: Vec<Block>) -> Vec<Block> {
        const KEEP: CowStr<'static> = CowStr::Borrowed("keep");
        const DROP: CowStr<'static> = CowStr::Borrowed("drop");
        let mut ret = Vec::new();
        for block in blocks {
            let currently_keeping = self.top().cur_keeping;
            if let Block::Heading(heading) = block {
                let cur_level = self.top().cur_level;
                let heading_keeps = heading.classes.contains(&KEEP);
                let heading_drops = heading.classes.contains(&DROP);
                match heading.level.cmp(&cur_level) {
                    Ordering::Greater => {
                        // Deeper, so if we're currently keeping we should check
                        // if this tells us to drop we should push a new drop context
                        if currently_keeping && heading_drops {
                            self.stack.push(KdMode {
                                keeping: false,
                                cur_keeping: false,
                                cur_level: heading.level,
                            });
                        } else {
                            // Wasn't a drop, so include this if we're keeping
                            if currently_keeping {
                                ret.push(Block::Heading(heading));
                            }
                        }
                    }
                    Ordering::Equal => {
                        // Same depth, flip back to "default" and then check for keep/drop
                        self.top().cur_keeping =
                            self.top().keeping && !heading_drops || heading_keeps;
                        if self.top().cur_keeping {
                            ret.push(Block::Heading(heading))
                        }
                    }
                    Ordering::Less => {
                        // Less deep, so we need to pop a level
                        self.stack.pop();
                    }
                }
            } else if currently_keeping {
                ret.push(self.fold_block(block));
            }
        }
        ret
    }

    fn fold_blocklist_push(&mut self, blocks: Vec<Block>) -> Vec<Block> {
        eprintln!("{blocks:?}");
        self.stack.push(KdMode {
            keeping: true,
            cur_keeping: true,
            cur_level: HeadingLevel::H1,
        });
        let ret = self.fold_blocklist(blocks);
        self.stack.pop();
        ret
    }
}

impl fold::MarkdownFold for KeepDrop {
    // We only need to fold wherever there a block lists

    fn fold_document(&mut self, document: Document) -> Document {
        Document {
            blocks: self.fold_blocklist(document.blocks),
        }
    }

    fn fold_block_quote(&mut self, block_quote: BlockQuote) -> BlockQuote {
        BlockQuote {
            kind: block_quote.kind,
            body: self.fold_blocklist_push(block_quote.body),
        }
    }

    fn fold_footnote_definition(
        &mut self,
        footnote_definition: FootnoteDefinition,
    ) -> FootnoteDefinition {
        FootnoteDefinition {
            label: footnote_definition.label,
            body: self.fold_blocklist_push(footnote_definition.body),
        }
    }

    fn fold_list_item(&mut self, list_item: ListItem) -> ListItem {
        match list_item {
            ListItem::Inline(vec) => fold_list_item(self, ListItem::Inline(vec)),
            ListItem::Block(vec) => ListItem::Block(self.fold_blocklist_push(vec)),
        }
    }
}

pub struct TodoFilter {
    level: HeadingLevel,
    processing: bool,
}

#[derive(Copy, Clone)]
enum ItemKind {
    PassThru,
    Unticked,
    Partial,
    WasPartial,
    Complete,
    WasComplete,
    Dropping,
    Dropped,
    Pausing,
    Paused,
}

impl ItemKind {
    fn cycle(self) -> Self {
        match self {
            ItemKind::PassThru => ItemKind::PassThru,
            ItemKind::Unticked => ItemKind::Unticked,
            ItemKind::Partial => ItemKind::WasPartial,
            ItemKind::WasPartial => ItemKind::Unticked,
            ItemKind::Complete => ItemKind::WasComplete,
            ItemKind::WasComplete => ItemKind::WasComplete,
            ItemKind::Dropping => ItemKind::Dropped,
            ItemKind::Dropped => ItemKind::Dropped,
            ItemKind::Pausing => ItemKind::Paused,
            ItemKind::Paused => ItemKind::Paused,
        }
    }

    fn implicit_space(self) -> bool {
        matches!(self, ItemKind::Unticked | ItemKind::Complete)
    }
}

impl From<char> for ItemKind {
    fn from(value: char) -> Self {
        match value {
            ' ' => Self::Unticked,
            '.' => Self::Partial,
            '-' => Self::WasPartial,
            'x' => Self::Complete,
            'F' => Self::WasComplete,
            'd' => Self::Dropping,
            'D' => Self::Dropped,
            'p' => Self::Pausing,
            'P' => Self::Paused,
            _ => Self::PassThru,
        }
    }
}

impl TodoFilter {
    const OPEN_SQUARE: CowStr<'static> = CowStr::Borrowed("[");
    const CLOSE_SQUARE: CowStr<'static> = CowStr::Borrowed("]");

    pub fn new() -> TodoFilter {
        TodoFilter {
            level: HeadingLevel::H1,
            processing: false,
        }
    }

    fn snaffle(&mut self, bits: &mut Vec<Inline>) -> ItemKind {
        if bits.is_empty() {
            return ItemKind::PassThru;
        }

        let kind = match bits.remove(0) {
            Inline::TasklistMarker(ticked) => {
                if ticked {
                    ItemKind::Complete
                } else {
                    ItemKind::Unticked
                }
            }
            Inline::Text(t) if t == Self::OPEN_SQUARE => {
                // OK, we had an open square, if there is a close square
                // we look deeper
                if let Some(Inline::Text(t)) = bits.get(1).cloned() {
                    if t == Self::CLOSE_SQUARE {
                        // Okay it's a close, finally we want text in the first position which
                        // is a single character
                        if let Some(Inline::Text(c)) = bits.first().cloned() {
                            if c.len() != 1 {
                                // Not one character
                                bits.insert(0, Inline::Text(Self::OPEN_SQUARE));
                                ItemKind::PassThru
                            } else {
                                let kind = ItemKind::from(c.chars().next().unwrap());
                                if matches!(kind, ItemKind::PassThru) {
                                    bits.insert(0, Inline::Text(Self::OPEN_SQUARE));
                                } else {
                                    // Remove c
                                    bits.remove(0);
                                    // remove ]
                                    bits.remove(0);
                                }
                                kind
                            }
                        } else {
                            // Not text
                            bits.insert(0, Inline::Text(Self::OPEN_SQUARE));
                            ItemKind::PassThru
                        }
                    } else {
                        // For whatever reason, the open isn't matched right
                        bits.insert(0, Inline::Text(Self::OPEN_SQUARE));
                        ItemKind::PassThru
                    }
                } else {
                    // Not even text
                    bits.insert(0, Inline::Text(Self::OPEN_SQUARE));
                    ItemKind::PassThru
                }
            }
            other => {
                bits.insert(0, other);
                ItemKind::PassThru
            }
        };

        // Now fold the rest of the list
        let new = bits.drain(..).map(|i| self.fold_inline(i)).collect();
        *bits = new;

        kind
    }

    fn snaffle_block(&mut self, bits: &mut Vec<Block>) -> ItemKind {
        if bits.is_empty() {
            return ItemKind::PassThru;
        }
        let mut first = bits.remove(0);
        let ret = match &mut first {
            Block::Paragraph(bits) => self.snaffle(&mut bits.body),
            _ => ItemKind::PassThru,
        };
        let new = Some(first)
            .into_iter()
            .chain(bits.drain(..).map(|b| self.fold_block(b)))
            .collect();
        *bits = new;
        ret
    }

    fn insert_kind(bits: &mut Vec<Inline>, old_kind: ItemKind) {
        let kind = old_kind.cycle();
        match (old_kind.implicit_space(), kind.implicit_space()) {
            (true, false) => {
                // We lose an implicit space, shove one in
                bits.insert(0, Inline::Text(CowStr::Borrowed(" ")));
            }
            (false, true) => {
                // We gain an implicit space, try and remove the space from bits[0]
                if let Some(Inline::Text(t)) = bits.get_mut(0) {
                    if &t[0..=0] == " " {
                        *t = CowStr::Boxed(t[1..].into());
                    }
                }
            }
            (true, true) | (false, false) => {
                // Nothing to do, nothing changed
            }
        }
        match kind {
            ItemKind::Partial | ItemKind::Dropping | ItemKind::Pausing | ItemKind::PassThru => {
                unreachable!()
            }

            ItemKind::Unticked => bits.insert(0, Inline::TasklistMarker(false)),
            ItemKind::Complete => bits.insert(0, Inline::TasklistMarker(true)),
            ItemKind::WasPartial => Self::insert_char(bits, '-'),
            ItemKind::WasComplete => Self::insert_char(bits, 'F'),
            ItemKind::Dropped => Self::insert_char(bits, 'D'),
            ItemKind::Paused => Self::insert_char(bits, 'P'),
        }
    }

    fn insert_char(bits: &mut Vec<Inline>, ch: char) {
        bits.insert(0, Inline::Text(Self::CLOSE_SQUARE));
        bits.insert(0, Inline::Text(CowStr::from(format!("{ch}"))));
        bits.insert(0, Inline::Text(Self::OPEN_SQUARE));
    }

    fn insert_kind_block(block: &mut Block, old_kind: ItemKind) {
        let Block::Paragraph(p) = block else {
            unreachable!()
        };
        Self::insert_kind(&mut p.body, old_kind);
    }

    fn adjust_item(&mut self, mut item: ListItem) -> Option<ListItem> {
        // Step one is to try and find out what this item even is.
        let kind = match &mut item {
            ListItem::Inline(vec) => self.snaffle(vec),
            ListItem::Block(vec) => self.snaffle_block(vec),
        };

        match kind {
            ItemKind::PassThru => Some(item),
            ItemKind::Dropped | ItemKind::WasComplete => None,
            _ => {
                match &mut item {
                    ListItem::Inline(vec) => Self::insert_kind(vec, kind),
                    ListItem::Block(vec) => Self::insert_kind_block(&mut vec[0], kind),
                };
                Some(item)
            }
        }
    }
}

impl Default for TodoFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownFold for TodoFilter {
    fn fold_list(&mut self, list: List) -> List {
        // We're folding the whole list because we might decide to not bother
        // with an item
        if self.processing {
            List {
                start: list.start,
                items: list
                    .items
                    .into_iter()
                    .filter_map(|i| self.adjust_item(i))
                    .collect(),
            }
        } else {
            fold_list(self, list)
        }
    }

    fn fold_document(&mut self, document: Document) -> Document {
        // Since we only care about the document level headers, this will do nicely
        let mut blocks = Vec::new();
        const TODO_CLASS: CowStr<'static> = CowStr::Borrowed("todo");

        for block in document.blocks {
            match block {
                Block::Heading(h) => {
                    if h.level <= self.level {
                        self.level = h.level;
                        self.processing = h.classes.contains(&TODO_CLASS);
                    } else {
                        // Deeper than 1, but switch to todo handling
                        if h.classes.contains(&TODO_CLASS) && !self.processing {
                            self.level = h.level;
                            self.processing = true;
                        }
                    }
                    blocks.push(Block::Heading(h));
                }
                _ => blocks.push(self.fold_block(block)),
            }
        }

        Document { blocks }
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use insta::assert_snapshot;
    use pulldown_cmark_ast::{fold::MarkdownFold, Document, ParseOptions};
    use rstest::rstest;

    use crate::{config::Configuration, filters::TodoFilter, markdown::MarkdownFile};

    use super::KeepDrop;

    const KEEP_DROP: &str = r###"

## Lower level entry first

maybe see this

# High level {.keep}

Expect to see this

# High level {.drop}

Won't see this

## Inner {.keep}

Don't expect this

# Outer once more

Maybe see this?

# Definitely have {.keep}

> We're keeping this blockquote
>
> # Drop from here? {.drop}
>
> Missing
>
> # See this bit again
>
> Woo

"###;

    #[rstest]
    #[case::keep(true)]
    #[case::drop(false)]
    fn keep_drop(#[case] keep: bool) {
        let md = Document::parse(KEEP_DROP, ParseOptions::all());
        let filtered = KeepDrop::new(keep).fold_document(md);
        let rmd = filtered.render(pulldown_cmark_ast::RenderOptions::default());
        assert_snapshot!(format!("keep_drop_{:?}", keep), rmd);
    }

    const TODO: &str = r###"
+++
+++
# no processing here

- [ ] Should be empty
- [.] Should be dot
- [-] Should be dash
- [x] Should be x
- [F] Should be F
- [d] Should be d
- [D] Should be D
- [p] Should be p
- [P] Should be P
- [/] Should be an unaffected slash

# TODOs here { .todo }

- [ ] Should remain empty
- [.] Should become dash
- [-] Should become empty
- [x] Should become F
- [F] Should vanish
- [d] Should become D
- [D] Should vanish
- [p] Should become P
- [P] Should remain P
- [/] Should be an unaffected slash

# No processing again

- [.] Should stay dot

## Sublist { .todo }

- [D] should vanish
  even though it's multiline
- [.] should become dash
  - Doing more stuff
  - [d] This should become D
- [x] Should become F

    "###;

    #[test]
    fn todo_processing() {
        let mut md = MarkdownFile::parse(Path::new(""), TODO).unwrap();
        md.filter_markdown(TodoFilter::new(), &Configuration::default());
        assert_snapshot!(md.markdown());
    }
}
