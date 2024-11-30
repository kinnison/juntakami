use pulldown_cmark::Alignment;
use pulldown_cmark::BlockQuoteKind;
use pulldown_cmark::CodeBlockKind;
use pulldown_cmark::CowStr;

use pulldown_cmark::HeadingLevel;
use pulldown_cmark::LinkType;
use pulldown_cmark::MetadataBlockKind;

#[derive(Debug, Clone)]
pub struct Document {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub enum Block {
    Metadata(MetadataBlock),
    Heading(Heading),
    Paragraph(Paragraph),
    BlockQuote(BlockQuote),
    CodeBlock(CodeBlock),
    HtmlBlock(HtmlBlock),
    FootnoteDefinition(FootnoteDefinition),
    Rule,
    List(List),
    DefinitionList(DefinitionList),
    Table(Table),
}

#[derive(Debug, Clone)]
pub struct MetadataBlock {
    pub kind: MetadataBlockKind,
    pub content: CowStr<'static>,
}

#[derive(Debug, Clone)]
pub struct Heading {
    pub level: HeadingLevel,
    pub id: Option<CowStr<'static>>,
    pub classes: Vec<CowStr<'static>>,
    pub attrs: Vec<(CowStr<'static>, Option<CowStr<'static>>)>,
    pub body: Vec<Inline>,
}

#[derive(Debug, Clone)]
pub struct Paragraph {
    pub body: Vec<Inline>,
}

#[derive(Debug, Clone)]
pub enum Inline {
    Text(CowStr<'static>),
    Image(Image),
    Link(Link),
    Html(CowStr<'static>),
    SoftBreak,
    HardBreak,
    InlineMath(CowStr<'static>),
    DisplayMath(CowStr<'static>),
    Code(CowStr<'static>),
    FootnoteReference(CowStr<'static>),
    TasklistMarker(bool),
    Emphasis(Vec<Inline>),
    Strong(Vec<Inline>),
    Strikethrough(Vec<Inline>),
}

#[derive(Debug, Clone)]
pub struct Image {
    pub link_type: LinkType,
    pub dest_url: CowStr<'static>,
    pub title: CowStr<'static>,
    pub id: CowStr<'static>,
    pub body: Vec<Inline>,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub link_type: LinkType,
    pub dest_url: CowStr<'static>,
    pub title: CowStr<'static>,
    pub id: CowStr<'static>,
    pub body: Vec<Inline>,
}

#[derive(Debug, Clone)]
pub struct BlockQuote {
    pub kind: Option<BlockQuoteKind>,
    pub body: Vec<Block>,
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub kind: CodeBlockKind<'static>,
    pub body: Vec<Inline>,
}

#[derive(Debug, Clone)]
pub struct HtmlBlock {
    pub body: Vec<CowStr<'static>>,
}

#[derive(Debug, Clone)]
pub struct FootnoteDefinition {
    pub label: CowStr<'static>,
    pub body: Vec<Block>,
}

#[derive(Debug, Clone)]
pub struct List {
    pub start: Option<u64>,
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub body: Vec<Inline>,
}

#[derive(Debug, Clone)]
pub struct DefinitionList {
    pub items: Vec<DefinitionItem>,
}

#[derive(Debug, Clone)]
pub struct DefinitionItem {
    pub title: Vec<Inline>,
    pub definitions: Vec<Vec<Inline>>,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub alignments: Vec<Alignment>,
    pub header: TableHead,
    pub rows: Vec<TableRow>,
}

#[derive(Debug, Clone)]
pub struct TableHead {
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone)]
pub struct TableCell {
    pub body: Vec<Inline>,
}
