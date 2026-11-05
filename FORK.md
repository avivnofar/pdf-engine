# How this fork works

pdf-engine = upstream PdfCraft, untouched where possible, plus a thin layer of our own. Keeping our
changes small and in our own files is what lets us keep pulling upstream's daily fixes.

## What is ours

| Path | What it is |
|---|---|
| `apps/pdf-engine-js/` | The JavaScript package: `src/core.rs` (plain Rust API, tested natively), `src/web.rs` (the `PdfEngine` class), `tests/smoke.mjs` (Node test of the built .wasm) |
| `crates/automation/src/memory.rs` | Open from bytes / save to bytes, so the engine works without a filesystem |
| `.github/workflows/js-package.yml` | Builds, tests and publishes the package to the `js-dist` branch |
| `docs/brand/*.svg` | Neutral placeholder art replacing the ArtCraft marks (their trademark terms require it); same file names so the UI crate needs no change |
| `README.md`, `FORK.md` | This documentation |

Lines we added to upstream files (each marked `pdf-engine fork`): one `mod` line in
`crates/automation/src/lib.rs`, one entry in `xtask/src/layers.rs`, and the regenerated
`ATTRIBUTION.toml` / `ATTRIBUTION.md` entries for the placeholder art.

## What is upstream's, and worth knowing

The engine is a Cargo workspace of focused crates. The ones you are most likely to reuse:

| Crate | Does |
|---|---|
| `cos` | Parse, repair and write PDF objects; incremental saves |
| `crypt` | PDF encryption, RC4 to AES-256, passwords and permissions |
| `organize` | Pages: combine, extract, split, rotate, bookmarks, labels |
| `annot` | Comments and markup with their appearances |
| `forms` | AcroForm fields: read, fill, flatten |
| `redact` | True redaction (removes the content, not just covers it) |
| `render` | Pages to pixels, text extraction in reading order |
| `ocr`, `sign`, `optimize`, `export`, `a11y` | OCR, signatures, compression, export, accessibility checks |
| `engine` | The façade over all of them: sessions, edits, undo, saving |
| `automation` | Every engine feature as a JSON tool (what the JS package exposes) |

`ui-egui`, `apps/pdfcraft` and `apps/pdfcraft-web` are upstream's desktop and web app. We don't
use them, but leave them in place so upstream merges stay clean. If you ever ship that app UI
yourself, rebrand it first: it still shows ArtCraft community links (`crates/engine/src/links.rs`)
and ships installers named PdfCraft (`packaging/`).

## Building the JS package

CI does it on every push to `main`. To build locally (Windows cmd, Rust installed from rustup.rs):

```cmd
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build apps\pdf-engine-js --target web --release --out-dir pkg
node apps\pdf-engine-js\tests\smoke.mjs
```

The package lands in `apps\pdf-engine-js\pkg`: `pdf_engine_js.js`, `pdf_engine_js_bg.wasm` and
TypeScript types in `pdf_engine_js.d.ts`.

## Pulling in upstream updates

```cmd
git remote add upstream https://github.com/storytold/pdfcraft.git
git fetch upstream
git checkout -b sync/upstream origin/main
git merge upstream/main
```

On a conflict in `README.md`, keep ours (`git checkout --ours README.md`). Everything else should
merge cleanly; if `crates/automation` changed its `open`/`save` internals, `memory.rs` is the one
file to adjust, and `cargo test -p pdfcraft-automation --lib memory` tells you. Open the sync as a
PR so the `js-package` workflow tests it before it reaches `main`. GitHub's **Sync fork** button
does the same merge when there are no conflicts.

## Upstream's CI in this fork

Upstream's own workflows (`ci.yml`, `sonarcloud.yml`, `release.yml` and the rest) use their shared
actions and secrets, so they fail here. Disable them once in the repository's **Actions** tab
(open each one, `…` → **Disable workflow**) and keep `js-package` enabled.
