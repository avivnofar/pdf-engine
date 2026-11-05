//! The `PdfEngine` class the npm package exports. Type conversion only; behaviour lives in
//! [`crate::core`]. JSON crosses the boundary through the browser's own `JSON`, so callers pass and
//! receive plain JavaScript objects, and binary data as `Uint8Array`.

use js_sys::{Array, Date, Object, Reflect, Uint8Array};
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::core::{Engine, Part};

fn to_js(v: &Value) -> Result<JsValue, JsError> {
    js_sys::JSON::parse(&v.to_string()).map_err(|_| JsError::new("could not convert the result"))
}

fn from_js(v: &JsValue) -> Result<Value, JsError> {
    if v.is_undefined() || v.is_null() {
        return Ok(Value::Object(Default::default()));
    }
    let text = js_sys::JSON::stringify(v).map_err(|_| JsError::new("arguments must be a plain object"))?;
    serde_json::from_str(&String::from(text)).map_err(|e| JsError::new(&e.to_string()))
}

fn err(e: String) -> JsError {
    JsError::new(&e)
}

fn set(obj: &Object, key: &str, value: &JsValue) -> Result<(), JsError> {
    Reflect::set(obj, &JsValue::from_str(key), value).map(|_| ()).map_err(|_| JsError::new("could not build the result"))
}

fn part_to_js(p: Part) -> Result<JsValue, JsError> {
    let obj = Object::new();
    match p {
        Part::Json(v) => set(&obj, "json", &to_js(&v)?)?,
        Part::Png { data, width, height } => {
            set(&obj, "png", &Uint8Array::from(data.as_slice()).into())?;
            set(&obj, "width", &width.into())?;
            set(&obj, "height", &height.into())?;
        }
    }
    Ok(obj.into())
}

fn now() -> i64 {
    (Date::now() / 1000.0) as i64
}

/// Seconds east of UTC for the browser's zone today (JavaScript reports minutes west).
fn utc_offset() -> i64 {
    -(Date::new_0().get_timezone_offset() as i64) * 60
}

/// A PDF workbench with no UI. Open documents by bytes, edit them with tools, save them back to
/// bytes. Pages are 1-based.
#[wasm_bindgen]
pub struct PdfEngine {
    inner: Engine,
}

#[wasm_bindgen]
impl PdfEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> PdfEngine {
        PdfEngine { inner: Engine::with_wall_clock(now, utc_offset) }
    }

    /// Every tool with its JSON Schema.
    pub fn tools() -> Result<JsValue, JsError> {
        to_js(&Engine::tools())
    }

    /// Open a PDF. Returns its summary; `summary.doc` is the id other calls take.
    pub fn open(&mut self, bytes: &[u8], name: Option<String>, password: Option<String>) -> Result<JsValue, JsError> {
        let v = self.inner.open(bytes.to_vec(), name.as_deref().unwrap_or(""), password.as_deref()).map_err(err)?;
        to_js(&v)
    }

    /// Run a tool and return its JSON result (`null` if it has none).
    pub fn run(&mut self, tool: &str, args: JsValue) -> Result<JsValue, JsError> {
        let v = self.inner.run(tool, &from_js(&args)?).map_err(err)?;
        to_js(&v)
    }

    /// Run a tool and return every part of its result: `[{ json }, { png, width, height }]`.
    pub fn call(&mut self, tool: &str, args: JsValue) -> Result<Array, JsError> {
        let parts = self.inner.call(tool, &from_js(&args)?).map_err(err)?;
        let out = Array::new();
        for p in parts {
            out.push(&part_to_js(p)?);
        }
        Ok(out)
    }

    /// Draw a page as PNG: `{ png: Uint8Array, width, height }`.
    #[wasm_bindgen(js_name = renderPage)]
    pub fn render_page(&mut self, doc: u32, page: u32, dpi: Option<f64>) -> Result<JsValue, JsError> {
        part_to_js(self.inner.render_page(doc.into(), page as usize, dpi).map_err(err)?)
    }

    /// The document's bytes with every edit applied.
    pub fn save(&mut self, doc: u32, full: Option<bool>, flatten_fill_sign: Option<bool>) -> Result<Uint8Array, JsError> {
        let bytes = self.inner.save(doc.into(), full.unwrap_or(false), flatten_fill_sign.unwrap_or(false)).map_err(err)?;
        Ok(Uint8Array::from(bytes.as_slice()))
    }

    /// Close a document; unsaved edits are refused unless `discardChanges` is true.
    pub fn close(&mut self, doc: u32, discard_changes: Option<bool>) -> Result<(), JsError> {
        self.inner.close(doc.into(), discard_changes.unwrap_or(false)).map_err(err)
    }

    /// Combine PDFs in order: `files` is an array of `{ name, bytes: Uint8Array }`.
    pub fn combine(&self, files: Array) -> Result<Uint8Array, JsError> {
        let mut list = Vec::new();
        for f in files.iter() {
            let name = Reflect::get(&f, &"name".into()).ok().and_then(|n| n.as_string()).unwrap_or_else(|| "document.pdf".into());
            let bytes = Reflect::get(&f, &"bytes".into()).map_err(|_| JsError::new("each file needs bytes"))?;
            if !bytes.is_instance_of::<Uint8Array>() {
                return Err(JsError::new("each file's bytes must be a Uint8Array"));
            }
            list.push((name, Uint8Array::from(bytes).to_vec()));
        }
        Ok(Uint8Array::from(self.inner.combine(list).map_err(err)?.as_slice()))
    }

    /// A new PDF with copies of these pages (1-based, in the order given).
    pub fn extract(&self, doc: u32, pages: Vec<u32>) -> Result<Uint8Array, JsError> {
        let pages: Vec<usize> = pages.into_iter().map(|p| p as usize).collect();
        Ok(Uint8Array::from(self.inner.extract(doc.into(), &pages).map_err(err)?.as_slice()))
    }

    /// Split every `n` pages: `[{ firstPage, lastPage, bytes }]`.
    #[wasm_bindgen(js_name = splitEvery)]
    pub fn split_every(&self, doc: u32, n: u32) -> Result<Array, JsError> {
        split_to_js(self.inner.split_every(doc.into(), n as usize).map_err(err)?)
    }

    /// Split before each of these pages (1-based): `[{ firstPage, lastPage, bytes }]`.
    #[wasm_bindgen(js_name = splitBefore)]
    pub fn split_before(&self, doc: u32, pages: Vec<u32>) -> Result<Array, JsError> {
        let pages: Vec<usize> = pages.into_iter().map(|p| p as usize).collect();
        split_to_js(self.inner.split_before(doc.into(), &pages).map_err(err)?)
    }
}

impl Default for PdfEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn split_to_js(parts: Vec<crate::core::SplitFile>) -> Result<Array, JsError> {
    let out = Array::new();
    for p in parts {
        let obj = Object::new();
        set(&obj, "firstPage", &(p.first_page as u32).into())?;
        set(&obj, "lastPage", &(p.last_page as u32).into())?;
        set(&obj, "bytes", &Uint8Array::from(p.bytes.as_slice()).into())?;
        out.push(&obj);
    }
    Ok(out)
}
