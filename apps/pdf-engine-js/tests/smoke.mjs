// End-to-end check of the built npm package in Node: the same .wasm a browser loads.
// Run after `wasm-pack build apps/pdf-engine-js --target web --release`:
//   node apps/pdf-engine-js/tests/smoke.mjs
import { readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { initSync, PdfEngine } from "../pkg/pdf_engine_js.js";

initSync({ module: readFileSync(new URL("../pkg/pdf_engine_js_bg.wasm", import.meta.url)) });

const pdf = new PdfEngine();

const tools = PdfEngine.tools();
assert.ok(tools.length > 50, `tool table: ${tools.length}`);

// Make a 4-page PDF in memory, save it to bytes, and open the bytes as a user's file would be.
const made = pdf.run("doc_create", { from: "blank", pages: 4 });
const original = pdf.save(made.doc, true);
assert.equal(new TextDecoder().decode(original.slice(0, 5)), "%PDF-");
pdf.close(made.doc);

const opened = pdf.open(original, "report.pdf");
assert.equal(opened.name, "report.pdf");
const doc = { doc: opened.doc };

pdf.run("page_rotate", { doc: doc.doc, pages: [2], degrees: 90 });

const shot = pdf.renderPage(doc.doc, 1, 36);
assert.ok(shot.png instanceof Uint8Array && shot.width > 0, "page renders to PNG");
assert.deepEqual([...shot.png.slice(1, 4)].map((c) => String.fromCharCode(c)).join(""), "PNG");

const edited = pdf.save(doc.doc);
assert.ok(edited.length > original.length, "incremental save appends");

const merged = pdf.combine([
  { name: "a.pdf", bytes: original },
  { name: "b.pdf", bytes: edited },
]);
const m = pdf.open(merged, "merged.pdf");
const parts = pdf.splitEvery(m.doc, 3);
assert.deepEqual(parts.map((p) => [p.firstPage, p.lastPage]), [[1, 3], [4, 6], [7, 8]]);
assert.ok(pdf.extract(m.doc, [8, 1]).length > 0);

assert.throws(() => pdf.open(new Uint8Array([1, 2, 3]), "bad.pdf"), "garbage is an error, not a crash");
assert.throws(() => pdf.extract(m.doc, [99]));

console.log("pdf-engine-js smoke test: ok");
