//! Folding for the AST

use pulldown_cmark::CowStr;

use crate::ast::*;

pub trait MarkdownFold {
    fn fold_document(&mut self, document: Document) -> Document {
        fold_document(self, document)
    }

    fn fold_block(&mut self, block: Block) -> Block {
        fold_block(self, block)
    }

    fn fold_inline(&mut self, inline: Inline) -> Inline {
        fold_inline(self, inline)
    }

    fn fold_rule(&mut self) {}
    fn fold_soft_break(&mut self) {}
    fn fold_hard_break(&mut self) {}

    fn fold_emphasis(&mut self, inlines: Vec<Inline>) -> Vec<Inline> {
        fold_inlines(self, inlines)
    }

    fn fold_strong(&mut self, inlines: Vec<Inline>) -> Vec<Inline> {
        fold_inlines(self, inlines)
    }

    fn fold_strikethrough(&mut self, inlines: Vec<Inline>) -> Vec<Inline> {
        fold_inlines(self, inlines)
    }

    fn fold_text(&mut self, text: CowStr<'static>) -> CowStr<'static> {
        text
    }

    fn fold_html(&mut self, html: CowStr<'static>) -> CowStr<'static> {
        html
    }

    fn fold_inline_math(&mut self, inline_math: CowStr<'static>) -> CowStr<'static> {
        inline_math
    }

    fn fold_display_math(&mut self, display_math: CowStr<'static>) -> CowStr<'static> {
        display_math
    }

    fn fold_code(&mut self, code: CowStr<'static>) -> CowStr<'static> {
        code
    }

    fn fold_tasklist_marker(&mut self, ticked: bool) -> bool {
        ticked
    }

    fn fold_footnote_reference(&mut self, footnote_reference: CowStr<'static>) -> CowStr<'static> {
        footnote_reference
    }

    fn fold_metadata_block(&mut self, metadata_block: MetadataBlock) -> MetadataBlock {
        metadata_block
    }

    fn fold_heading(&mut self, heading: Heading) -> Heading {
        fold_heading(self, heading)
    }

    fn fold_paragraph(&mut self, paragraph: Paragraph) -> Paragraph {
        fold_paragraph(self, paragraph)
    }

    fn fold_block_quote(&mut self, block_quote: BlockQuote) -> BlockQuote {
        fold_block_quote(self, block_quote)
    }

    fn fold_code_block(&mut self, code_block: CodeBlock) -> CodeBlock {
        fold_code_block(self, code_block)
    }

    fn fold_html_block(&mut self, html_block: HtmlBlock) -> HtmlBlock {
        html_block
    }

    fn fold_footnote_definition(
        &mut self,
        footnote_definition: FootnoteDefinition,
    ) -> FootnoteDefinition {
        fold_footnote_definition(self, footnote_definition)
    }

    fn fold_list(&mut self, list: List) -> List {
        fold_list(self, list)
    }

    fn fold_list_item(&mut self, list_item: ListItem) -> ListItem {
        fold_list_item(self, list_item)
    }

    fn fold_definition_list(&mut self, definition_list: DefinitionList) -> DefinitionList {
        fold_definition_list(self, definition_list)
    }

    fn fold_definition_item(&mut self, definition_item: DefinitionItem) -> DefinitionItem {
        fold_definition_item(self, definition_item)
    }

    fn fold_definition_definition(
        &mut self,
        definition_definition: DefinitionDefinition,
    ) -> DefinitionDefinition {
        fold_definition_definition(self, definition_definition)
    }

    fn fold_table(&mut self, table: Table) -> Table {
        fold_table(self, table)
    }

    fn fold_table_head(&mut self, table_head: TableHead) -> TableHead {
        fold_table_head(self, table_head)
    }

    fn fold_table_row(&mut self, table_row: TableRow) -> TableRow {
        fold_table_row(self, table_row)
    }

    fn fold_table_cell(&mut self, table_cell: TableCell) -> TableCell {
        fold_table_cell(self, table_cell)
    }

    fn fold_image(&mut self, image: Image) -> Image {
        fold_image(self, image)
    }

    fn fold_link(&mut self, link: Link) -> Link {
        fold_link(self, link)
    }
}

pub fn fold_inlines<F: MarkdownFold + ?Sized>(folder: &mut F, inlines: Vec<Inline>) -> Vec<Inline> {
    inlines.into_iter().map(|i| folder.fold_inline(i)).collect()
}

pub fn fold_blocks<F: MarkdownFold + ?Sized>(folder: &mut F, blocks: Vec<Block>) -> Vec<Block> {
    blocks.into_iter().map(|b| folder.fold_block(b)).collect()
}

pub fn fold_document<F: MarkdownFold + ?Sized>(folder: &mut F, document: Document) -> Document {
    Document {
        blocks: fold_blocks(folder, document.blocks),
    }
}

pub fn fold_block<F: MarkdownFold + ?Sized>(folder: &mut F, block: Block) -> Block {
    match block {
        Block::Metadata(metadata_block) => {
            Block::Metadata(folder.fold_metadata_block(metadata_block))
        }
        Block::Heading(heading) => Block::Heading(folder.fold_heading(heading)),
        Block::Paragraph(paragraph) => Block::Paragraph(folder.fold_paragraph(paragraph)),
        Block::BlockQuote(block_quote) => Block::BlockQuote(folder.fold_block_quote(block_quote)),
        Block::CodeBlock(code_block) => Block::CodeBlock(folder.fold_code_block(code_block)),
        Block::HtmlBlock(html_block) => Block::HtmlBlock(folder.fold_html_block(html_block)),
        Block::FootnoteDefinition(footnote_definition) => {
            Block::FootnoteDefinition(folder.fold_footnote_definition(footnote_definition))
        }
        Block::List(list) => Block::List(folder.fold_list(list)),
        Block::DefinitionList(definition_list) => {
            Block::DefinitionList(folder.fold_definition_list(definition_list))
        }
        Block::Table(table) => Block::Table(folder.fold_table(table)),
        Block::Rule => {
            folder.fold_rule();
            Block::Rule
        }
    }
}

pub fn fold_inline<F: MarkdownFold + ?Sized>(folder: &mut F, inline: Inline) -> Inline {
    match inline {
        Inline::SoftBreak => {
            folder.fold_soft_break();
            Inline::SoftBreak
        }
        Inline::HardBreak => {
            folder.fold_hard_break();
            Inline::HardBreak
        }
        Inline::TasklistMarker(ticked) => {
            Inline::TasklistMarker(folder.fold_tasklist_marker(ticked))
        }
        Inline::Image(image) => Inline::Image(folder.fold_image(image)),
        Inline::Link(link) => Inline::Link(folder.fold_link(link)),

        Inline::Text(text) => Inline::Text(folder.fold_text(text)),
        Inline::Html(html) => Inline::Html(folder.fold_html(html)),
        Inline::InlineMath(inline_math) => Inline::InlineMath(folder.fold_inline_math(inline_math)),
        Inline::DisplayMath(display_math) => {
            Inline::DisplayMath(folder.fold_display_math(display_math))
        }
        Inline::Code(code) => Inline::Code(folder.fold_code(code)),
        Inline::FootnoteReference(footnote_reference) => {
            Inline::FootnoteReference(folder.fold_footnote_reference(footnote_reference))
        }

        Inline::Emphasis(inlines) => Inline::Emphasis(folder.fold_emphasis(inlines)),
        Inline::Strong(inlines) => Inline::Strong(folder.fold_strong(inlines)),
        Inline::Strikethrough(inlines) => Inline::Strikethrough(folder.fold_strikethrough(inlines)),
    }
}

pub fn fold_heading<F: MarkdownFold + ?Sized>(folder: &mut F, heading: Heading) -> Heading {
    Heading {
        level: heading.level,
        id: heading.id,
        classes: heading.classes,
        attrs: heading.attrs,
        body: fold_inlines(folder, heading.body),
    }
}

pub fn fold_paragraph<F: MarkdownFold + ?Sized>(folder: &mut F, paragraph: Paragraph) -> Paragraph {
    Paragraph {
        body: fold_inlines(folder, paragraph.body),
    }
}

pub fn fold_link<F: MarkdownFold + ?Sized>(folder: &mut F, link: Link) -> Link {
    Link {
        link_type: link.link_type,
        dest_url: link.dest_url,
        title: link.title,
        id: link.id,
        body: fold_inlines(folder, link.body),
    }
}

pub fn fold_image<F: MarkdownFold + ?Sized>(folder: &mut F, link: Image) -> Image {
    Image {
        link_type: link.link_type,
        dest_url: link.dest_url,
        title: link.title,
        id: link.id,
        body: fold_inlines(folder, link.body),
    }
}

pub fn fold_block_quote<F: MarkdownFold + ?Sized>(
    folder: &mut F,
    block_quote: BlockQuote,
) -> BlockQuote {
    BlockQuote {
        kind: block_quote.kind,
        body: fold_blocks(folder, block_quote.body),
    }
}

pub fn fold_code_block<F: MarkdownFold + ?Sized>(
    folder: &mut F,
    code_block: CodeBlock,
) -> CodeBlock {
    CodeBlock {
        kind: code_block.kind,
        body: fold_inlines(folder, code_block.body),
    }
}

pub fn fold_footnote_definition<F: MarkdownFold + ?Sized>(
    folder: &mut F,
    footnote_definition: FootnoteDefinition,
) -> FootnoteDefinition {
    FootnoteDefinition {
        label: footnote_definition.label,
        body: fold_blocks(folder, footnote_definition.body),
    }
}

pub fn fold_list<F: MarkdownFold + ?Sized>(folder: &mut F, list: List) -> List {
    List {
        start: list.start,
        items: list
            .items
            .into_iter()
            .map(|li| folder.fold_list_item(li))
            .collect(),
    }
}

pub fn fold_list_item<F: MarkdownFold + ?Sized>(folder: &mut F, list_item: ListItem) -> ListItem {
    ListItem {
        body: fold_inlines(folder, list_item.body),
    }
}

pub fn fold_definition_list<F: MarkdownFold + ?Sized>(
    folder: &mut F,
    definition_list: DefinitionList,
) -> DefinitionList {
    DefinitionList {
        items: definition_list
            .items
            .into_iter()
            .map(|di| folder.fold_definition_item(di))
            .collect(),
    }
}

pub fn fold_definition_item<F: MarkdownFold + ?Sized>(
    folder: &mut F,
    definition_item: DefinitionItem,
) -> DefinitionItem {
    DefinitionItem {
        title: fold_inlines(folder, definition_item.title),
        definitions: definition_item
            .definitions
            .into_iter()
            .map(|dd| folder.fold_definition_definition(dd))
            .collect(),
    }
}

pub fn fold_definition_definition<F: MarkdownFold + ?Sized>(
    folder: &mut F,
    definition_definition: DefinitionDefinition,
) -> DefinitionDefinition {
    DefinitionDefinition {
        body: fold_inlines(folder, definition_definition.body),
    }
}

pub fn fold_table<F: MarkdownFold + ?Sized>(folder: &mut F, table: Table) -> Table {
    Table {
        alignments: table.alignments,
        header: folder.fold_table_head(table.header),
        rows: table
            .rows
            .into_iter()
            .map(|tr| folder.fold_table_row(tr))
            .collect(),
    }
}

pub fn fold_table_head<F: MarkdownFold + ?Sized>(
    folder: &mut F,
    table_head: TableHead,
) -> TableHead {
    TableHead {
        cells: table_head
            .cells
            .into_iter()
            .map(|c| folder.fold_table_cell(c))
            .collect(),
    }
}

pub fn fold_table_row<F: MarkdownFold + ?Sized>(folder: &mut F, table_row: TableRow) -> TableRow {
    TableRow {
        cells: table_row
            .cells
            .into_iter()
            .map(|c| folder.fold_table_cell(c))
            .collect(),
    }
}

pub fn fold_table_cell<F: MarkdownFold + ?Sized>(
    folder: &mut F,
    table_cell: TableCell,
) -> TableCell {
    TableCell {
        body: fold_inlines(folder, table_cell.body),
    }
}
