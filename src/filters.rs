//! Filters for markdown trees

use std::cmp::Ordering;

use pulldown_cmark_ast::{
    fold::{self, fold_list_item, MarkdownFold},
    Block, BlockQuote, CowStr, Document, FootnoteDefinition, HeadingLevel, ListItem,
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

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use pulldown_cmark_ast::{fold::MarkdownFold, Document, ParseOptions};
    use rstest::rstest;

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
}
