//! Markdown files as used by Jun Takami
//!
//! Markdown files have frontmatter presented as TOML between +++s
//! and then markdown in the main body of the file.  The markdown is
//! always stored as the raw bytes, and we only adjust it if we are
//! running a transform for some reason.
//!
//! The frontmatter is stored a the TOML value so that editing operations
//! can be performed with relative ease and the frontmatter be reserialised
//!

use std::path::{Path, PathBuf};

use eyre::{bail, Context, Result};
use once_cell::sync::Lazy;
use pulldown_cmark_ast::{fold::MarkdownFold, Document, ParseOptions, RenderOptions};
use regex::Regex;
use toml_edit::Item;

use crate::config::Configuration;

pub struct MarkdownFile {
    origin: PathBuf,
    frontmatter: toml_edit::DocumentMut,
    markdown: String,
}

static TOML_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^[[:space:]]*\+\+\+(?:\r?\n)((?s).*?(?-s))\+\+\+[[:blank:]]*(?:$|(?:\r?\n((?s).*(?-s))$))",
    )
    .unwrap()
});

static LIST_TIDY_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\-\*] +)\\\[\\?(.)\\\] ").unwrap());

impl MarkdownFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        fn _load(path: &Path) -> Result<MarkdownFile> {
            let body = std::fs::read_to_string(path)
                .with_context(|| format!("Trying to load Markdown file: {}", path.display()))?;
            MarkdownFile::parse(path, &body)
        }
        _load(path.as_ref())
    }

    pub fn parse(origin: &Path, body: &str) -> Result<Self> {
        let Some(caps) = TOML_RE.captures(body) else {
            bail!(
                "Unable to extract frontmatter from file: {}",
                origin.display()
            );
        };
        // If we matched then caps[1] is the frontmatter and caps[2] is the markdown
        let frontmatter = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let markdown = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_owned();
        let frontmatter = frontmatter
            .parse()
            .with_context(|| format!("Trying to parse frontmatter from: {}", origin.display()))?;
        let origin = origin.to_owned();
        Ok(MarkdownFile {
            origin,
            frontmatter,
            markdown,
        })
    }

    pub fn origin(&self) -> &Path {
        &self.origin
    }

    pub fn markdown(&self) -> &str {
        &self.markdown
    }

    pub fn render_raw(&self) -> String {
        let mut frontmatter = self.frontmatter.to_string();
        if !frontmatter.ends_with('\n') {
            frontmatter.push('\n');
        }
        format!("+++\n{frontmatter}+++\n{}", self.markdown)
    }

    pub fn write_raw(&self, target: Option<impl AsRef<Path>>) -> Result<()> {
        let target = target.as_ref().map(|p| p.as_ref()).unwrap_or(&self.origin);
        let body = self.render_raw();
        std::fs::write(target, &body)
            .with_context(|| format!("Attempting to write to: {}", target.display()))
    }

    pub fn new_log_entry() -> MarkdownFile {
        // We create a new log entry with a default format
        let body = include_str!("template-log.md");
        Self::parse(Path::new("builtin-template"), body).unwrap()
    }

    pub fn set_title(&mut self, title: &str) {
        *self
            .frontmatter
            .entry("title")
            .or_insert(toml_edit::Item::None) = title.into();
    }

    pub fn set_created(&mut self, created: &str) {
        *self
            .frontmatter
            .entry("created")
            .or_insert(toml_edit::Item::None) = created.into();
    }

    pub fn set_author(&mut self, author: &str) {
        *self
            .frontmatter
            .entry("author")
            .or_insert(toml_edit::Item::None) = author.into();
    }

    pub fn keep_drop(&self) -> bool {
        self.frontmatter
            .get("keep")
            .and_then(Item::as_bool)
            .unwrap_or(false)
    }

    pub fn filter_markdown(&mut self, mut filter: impl MarkdownFold, config: &Configuration) {
        let doc = Document::parse(&self.markdown, parse_opts());
        let filtered = filter.fold_document(doc);
        self.markdown = LIST_TIDY_RE
            .replace_all(&filtered.render(render_opts(config)), "$1[$2] ")
            .into_owned();
    }
}

fn parse_opts() -> ParseOptions {
    ParseOptions::all().intersection(ParseOptions::ENABLE_SMART_PUNCTUATION.complement())
}

fn render_opts(config: &Configuration) -> RenderOptions<'_> {
    RenderOptions {
        list_token: config.list_char(),
        strong_token: "**",
        emphasis_token: '_',
        ordered_list_token: '.',
        code_block_token: '`',
        increment_ordered_list_bullets: true,
        ..Default::default()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn roundtrip() {
        let input = r#"+++
cake = "delicious"
# Foo!
ages = [ 1, 2, 3,4, 5 ]
+++

# Everybody loves parfait!
"#;
        let md = MarkdownFile::parse(Path::new(""), input).unwrap();
        let output = md.render_raw();
        assert_eq!(input, output);
    }
}
