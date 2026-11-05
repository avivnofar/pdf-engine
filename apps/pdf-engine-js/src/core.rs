//! The engine behind the JavaScript API, in plain Rust so it can be tested natively.
//!
//! Documents are addressed by the id [`Engine::open`] returns, and pages are **1-based**, as in
//! every PdfCraft tool. Errors are human-readable strings, ready to show to a user.

use std::sync::Arc;

use pdfcraft_automation::{Automation, Content};
use pdfcraft_engine::DocId;
use pdfcraft_organize::SplitBy;
use serde_json::{Value, json};

/// The result type of every call: a message on failure.
pub type Result<T> = std::result::Result<T, String>;

/// One part of a tool's result.
#[derive(Debug, Clone, PartialEq)]
pub enum Part {
    Json(Value),
    Png { data: Vec<u8>, width: u32, height: u32 },
}

/// One file of a split: its first and last page (1-based, inclusive) and its PDF bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitFile {
    pub first_page: usize,
    pub last_page: usize,
    pub bytes: Vec<u8>,
}

/// Most files one combine accepts, a guard against a runaway caller.
pub const MAX_COMBINE_FILES: usize = 1000;

/// A PDF workbench with no UI: any number of open documents, each edited in memory until saved.
pub struct Engine {
    auto: Automation,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self { auto: Automation::new() }
    }

    /// An engine whose saves and Fill & Sign dates use this clock (UTC seconds since the epoch)
    /// and time zone (seconds east of UTC). The browser build has no clock of its own.
    pub fn with_wall_clock(clock: fn() -> i64, utc_offset: fn() -> i64) -> Self {
        Self { auto: Automation::new().with_wall_clock(clock, utc_offset) }
    }

    /// Every tool, with its JSON Schema: `[{ name, title, description, input_schema, read_only,
    /// destructive }]`. Tools that read or write file paths need a filesystem, so in the browser
    /// use the bytes-based methods of this type instead (open, save, combine, extract, split).
    pub fn tools() -> Value {
        Value::Array(
            pdfcraft_automation::tools()
                .into_iter()
                .map(|t| {
                    json!({
                        "name": t.name, "title": t.title, "description": t.description,
                        "input_schema": t.input_schema, "read_only": t.read_only, "destructive": t.destructive,
                    })
                })
                .collect(),
        )
    }

    /// Open a PDF from its bytes. Returns the document summary; its `doc` field is the id that
    /// every other call takes (as in the `doc_open` tool).
    pub fn open(&mut self, bytes: Vec<u8>, name: &str, password: Option<&str>) -> Result<Value> {
        let name = if name.trim().is_empty() { "document.pdf" } else { name };
        self.auto.open_bytes(name, bytes, password).map_err(|e| e.to_string())
    }

    /// Run any tool by name with a JSON object of arguments (see [`Engine::tools`]).
    pub fn call(&mut self, tool: &str, args: &Value) -> Result<Vec<Part>> {
        let parts = self.auto.call(tool, args).map_err(|e| e.to_string())?;
        Ok(parts
            .into_iter()
            .map(|p| match p {
                Content::Json(v) => Part::Json(v),
                Content::Png { data, width, height } => Part::Png { data, width, height },
            })
            .collect())
    }

    /// Run a tool and return its JSON result alone (`null` when it has none).
    pub fn run(&mut self, tool: &str, args: &Value) -> Result<Value> {
        Ok(self.call(tool, args)?.into_iter().find_map(|p| if let Part::Json(v) = p { Some(v) } else { None }).unwrap_or(Value::Null))
    }

    /// The page drawn as a PNG image at `dpi` (the engine caps it at 600).
    pub fn render_page(&mut self, doc: u64, page: usize, dpi: Option<f64>) -> Result<Part> {
        let mut args = json!({ "doc": doc, "page": page });
        if let Some(dpi) = dpi {
            args["dpi"] = json!(dpi);
        }
        self.call("page_render", &args)?.into_iter().find(|p| matches!(p, Part::Png { .. })).ok_or_else(|| "the page could not be drawn".to_owned())
    }

    /// The document's bytes with all edits applied. `full: false` appends an incremental update
    /// and keeps the original bytes intact; `full: true` rewrites the whole file (smaller, and
    /// what to use after removing content you don't want recoverable).
    pub fn save(&mut self, doc: u64, full: bool, flatten_fill_sign: bool) -> Result<Vec<u8>> {
        self.auto.save_to_bytes(doc, full, flatten_fill_sign).map_err(|e| e.to_string())
    }

    /// Close a document. Unsaved edits are refused unless `discard_changes` is set.
    pub fn close(&mut self, doc: u64, discard_changes: bool) -> Result<()> {
        self.run("doc_close", &json!({ "doc": doc, "discard_changes": discard_changes })).map(|_| ())
    }

    /// Combine whole PDFs, in order, into one (one bookmark per file, named after it).
    pub fn combine(&self, files: Vec<(String, Vec<u8>)>) -> Result<Vec<u8>> {
        if files.is_empty() {
            return Err("give at least one file to combine".into());
        }
        if files.len() > MAX_COMBINE_FILES {
            return Err(format!("at most {MAX_COMBINE_FILES} files can be combined at once"));
        }
        let sources: Vec<(String, Arc<Vec<u8>>)> = files.into_iter().map(|(name, bytes)| (name, Arc::new(bytes))).collect();
        self.auto.session().combine(&sources).map(Arc::unwrap_or_clone).map_err(|e| e.to_string())
    }

    /// A new PDF holding copies of `pages` (1-based) of an open document, in the order given.
    pub fn extract(&self, doc: u64, pages: &[usize]) -> Result<Vec<u8>> {
        let pages = self.zero_based(doc, pages)?;
        self.auto.session().extract(DocId(doc), &pages).map(Arc::unwrap_or_clone).map_err(|e| e.to_string())
    }

    /// Split an open document into new PDFs every `every` pages (`every` ≥ 1).
    pub fn split_every(&self, doc: u64, every: usize) -> Result<Vec<SplitFile>> {
        if every == 0 {
            return Err("split every 1 page or more".into());
        }
        self.split(doc, &SplitBy::PageCount(every))
    }

    /// Split an open document before each of these pages (1-based): `[3, 7]` makes pages 1–2,
    /// 3–6 and 7 to the end.
    pub fn split_before(&self, doc: u64, pages: &[usize]) -> Result<Vec<SplitFile>> {
        let before = self.zero_based(doc, pages)?;
        self.split(doc, &SplitBy::Before(before))
    }

    fn split(&self, doc: u64, by: &SplitBy) -> Result<Vec<SplitFile>> {
        let parts = self.auto.session().split(DocId(doc), by).map_err(|e| e.to_string())?;
        Ok(parts.into_iter().map(|(first_page, last_page, bytes)| SplitFile { first_page, last_page, bytes: Arc::unwrap_or_clone(bytes) }).collect())
    }

    /// 1-based page numbers → 0-based indices, each checked against the document's page count.
    fn zero_based(&self, doc: u64, pages: &[usize]) -> Result<Vec<usize>> {
        let count = self.auto.session().get(DocId(doc)).ok_or_else(|| format!("no open document with id {doc}"))?.info.pages.len();
        if pages.is_empty() {
            return Err("give at least one page".into());
        }
        pages.iter().map(|&p| if (1..=count).contains(&p) { Ok(p - 1) } else { Err(format!("page {p} is outside 1–{count}")) }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank(pages: usize) -> Vec<u8> {
        let e = Engine::new();
        e.auto.session().create_blank(612.0, 792.0, pages).map(Arc::unwrap_or_clone).unwrap_or_default()
    }

    fn page_count(e: &mut Engine, bytes: Vec<u8>) -> u64 {
        let doc = e.open(bytes, "check.pdf", None).unwrap_or_default();
        doc["pages"].as_u64().or_else(|| doc["page_count"].as_u64()).unwrap_or(0)
    }

    #[test]
    fn tools_lists_every_tool_with_a_schema() {
        let tools = Engine::tools();
        let list = tools.as_array().cloned().unwrap_or_default();
        assert!(list.len() > 50, "the full tool table, got {}", list.len());
        assert!(list.iter().any(|t| t["name"] == "page_rotate" && t["input_schema"].is_object()));
    }

    #[test]
    fn open_edit_render_save() {
        let mut e = Engine::new();
        let doc = e.open(blank(2), "", None).unwrap_or_default();
        let id = doc["doc"].as_u64().unwrap_or(0);
        assert!(id > 0, "{doc}");
        assert_eq!(doc["name"], "document.pdf", "an empty name gets a default");
        e.run("page_rotate", &json!({ "doc": id, "pages": [2], "degrees": 90 })).unwrap_or_default();
        let png = e.render_page(id, 1, Some(36.0));
        assert!(
            matches!(&png, Ok(Part::Png { data, width, height }) if data.starts_with(b"\x89PNG") && *width > 0 && *height > 0),
            "expected a PNG, got {png:?}"
        );
        let saved = e.save(id, true, false).unwrap_or_default();
        assert!(saved.starts_with(b"%PDF"));
        assert!(e.close(id, false).is_ok(), "a saved document closes without discarding");
    }

    #[test]
    fn unsaved_edits_are_not_lost_on_close() {
        let mut e = Engine::new();
        let id = e.open(blank(1), "a.pdf", None).unwrap_or_default()["doc"].as_u64().unwrap_or(0);
        e.run("page_rotate", &json!({ "doc": id, "degrees": 90 })).unwrap_or_default();
        assert!(e.close(id, false).is_err());
        assert!(e.close(id, true).is_ok());
    }

    #[test]
    fn combine_extract_split() {
        let mut e = Engine::new();
        let merged = e.combine(vec![("a.pdf".into(), blank(2)), ("b.pdf".into(), blank(3))]);
        assert!(merged.is_ok(), "{merged:?}");
        let merged = merged.unwrap_or_default();
        assert_eq!(page_count(&mut e, merged.clone()), 5);

        let id = e.open(merged, "m.pdf", None).unwrap_or_default()["doc"].as_u64().unwrap_or(0);
        let two = e.extract(id, &[5, 1]).unwrap_or_default();
        assert_eq!(page_count(&mut e, two), 2);

        let parts = e.split_every(id, 2).unwrap_or_default();
        assert_eq!(parts.iter().map(|p| (p.first_page, p.last_page)).collect::<Vec<_>>(), vec![(1, 2), (3, 4), (5, 5)]);
        let parts = e.split_before(id, &[4]).unwrap_or_default();
        assert_eq!(parts.iter().map(|p| (p.first_page, p.last_page)).collect::<Vec<_>>(), vec![(1, 3), (4, 5)]);
    }

    #[test]
    fn bad_input_is_an_error_not_a_crash() {
        let mut e = Engine::new();
        assert!(e.open(b"%PDF-1.7 garbage".to_vec(), "x.pdf", None).is_err() || e.open(Vec::new(), "x.pdf", None).is_err());
        assert!(e.combine(Vec::new()).is_err());
        assert!(e.combine(vec![("x.pdf".into(), b"nope".to_vec())]).is_err());
        let id = e.open(blank(2), "a.pdf", None).unwrap_or_default()["doc"].as_u64().unwrap_or(0);
        assert!(e.extract(id, &[0]).is_err());
        assert!(e.extract(id, &[3]).is_err());
        assert!(e.extract(id, &[]).is_err());
        assert!(e.extract(999, &[1]).is_err());
        assert!(e.split_every(id, 0).is_err());
        assert!(e.render_page(id, 9, None).is_err());
        assert!(e.run("no_such_tool", &json!({})).is_err());
    }
}
