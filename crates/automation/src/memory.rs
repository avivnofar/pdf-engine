//! In-memory documents: open from bytes and save to bytes, for hosts without a filesystem such as
//! the browser build (`crates/js`, the pdf-engine fork's JavaScript package).
//!
//! Every other tool works unchanged on a document opened here, because tools address documents by
//! id. Only tools that read or write paths (`doc_open`, `doc_save`, `doc_combine`, `doc_split`, a
//! `page_render` with `out`) need a filesystem; the bytes-based counterparts live here and in
//! [`pdfcraft_engine::Session`] (`combine`, `extract`, `split`).
//!
//! Kept in its own file, reached by one `mod` line in `lib.rs`, so merging upstream PdfCraft into
//! the fork touches as little shared code as possible.

use std::sync::Arc;

use pdfcraft_engine::Edit;
use serde_json::{Value, json};

use super::Automation;
use super::error::{Result, ToolError, failed};
use super::json::{Args, summary};

/// The largest document accepted from memory: the same cap the file reader applies (512 MiB).
pub const MAX_BYTES: usize = 512 * 1024 * 1024;

impl Automation {
    /// Open a PDF held in memory under the display name `name`. Returns the same summary as the
    /// `doc_open` tool (its `id` is the `doc` every other tool takes). The document has no path,
    /// so it can only be saved with [`Automation::save_to_bytes`].
    pub fn open_bytes(&mut self, name: &str, bytes: Vec<u8>, password: Option<&str>) -> Result<Value> {
        if bytes.len() > MAX_BYTES {
            return Err(ToolError::InvalidArgs(format!("the document is larger than {} MiB", MAX_BYTES / (1024 * 1024))));
        }
        let id = self.session.open(name.to_owned(), None, Arc::new(bytes), password).map_err(failed)?;
        let doc = self.session.get(id).ok_or_else(|| failed("the document vanished"))?;
        Ok(summary(doc))
    }

    /// The document's bytes with every edit applied, validated the same way `doc_save` validates
    /// before writing, and recorded as saved (the next save appends only newer edits).
    ///
    /// `full: false` appends an incremental update, so the original bytes stay intact (the engine
    /// still rewrites in full when it must, after redaction, sanitizing or an encryption change).
    /// `flatten_fill_sign` flattens Fill & Sign marks into page content first, as in `doc_save`.
    pub fn save_to_bytes(&mut self, doc: u64, full: bool, flatten_fill_sign: bool) -> Result<Vec<u8>> {
        let v = json!({ "doc": doc });
        let a = Args(&v);
        let id = self.doc(&a)?.id;
        if flatten_fill_sign {
            self.apply(&a, Edit::FlattenFillSign)?;
        }
        let bytes = if full { self.session.save_full_bytes(id) } else { self.session.save_bytes(id) }.map_err(failed)?;
        self.session.check_save(id, &bytes).map_err(failed)?;
        self.session.mark_saved(id, bytes.clone(), None).map_err(failed)?;
        Ok(Arc::unwrap_or_clone(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A one-page PDF built from the engine itself, so the test needs no fixture file.
    fn blank_pdf(a: &Automation) -> Vec<u8> {
        let bytes = a.session().create_blank(612.0, 792.0, 1).unwrap_or_default();
        bytes.as_ref().clone()
    }

    #[test]
    fn open_edit_save_round_trip_in_memory() {
        let mut a = Automation::new().with_clock(|| 0);
        let pdf = blank_pdf(&a);
        assert!(pdf.starts_with(b"%PDF"), "the engine should build a blank PDF");
        let opened = a.open_bytes("blank.pdf", pdf.clone(), None).unwrap_or_default();
        let id = opened["doc"].as_u64().unwrap_or(0);
        assert!(id > 0, "open returns a document id: {opened}");
        a.call("page_rotate", &json!({ "doc": id, "pages": [1], "degrees": 90 })).unwrap_or_default();
        let saved = a.save_to_bytes(id, false, false).unwrap_or_default();
        assert!(saved.len() > pdf.len() && saved.starts_with(&pdf), "an incremental save appends to the original bytes");
        let mut b = Automation::new();
        let again = b.open_bytes("saved.pdf", saved, None).unwrap_or_default();
        let info = b.call("doc_info", &json!({ "doc": again["doc"] })).unwrap_or_default();
        assert!(format!("{info:?}").contains("90"), "the rotation survives the save: {info:?}");
    }

    #[test]
    fn refuses_what_is_not_a_pdf_and_unknown_ids() {
        let mut a = Automation::new();
        assert!(a.open_bytes("x.pdf", b"not a pdf".to_vec(), None).is_err());
        assert!(a.save_to_bytes(42, false, false).is_err());
    }
}
