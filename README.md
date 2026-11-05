# pdf-engine

A PDF engine you can drop into any website or tool: open, inspect, edit, render, combine, split,
protect and save PDFs **in the browser or in Node**, with no server and no UI of its own.

pdf-engine is a fork of [PdfCraft](https://github.com/storytold/pdfcraft) by the ArtCraft team, a
clean-room, open-source PDF reader and editor written in Rust. This fork keeps PdfCraft's engine
crates as they are and adds a JavaScript package on top of them. It is not affiliated with or
endorsed by the ArtCraft team.

## Use it from JavaScript

The `js-package` workflow builds the package on every change to `main` and publishes it to the
`js-dist` branch, so a page can load it straight from a CDN:

```html
<script type="module">
  import init, { PdfEngine } from "https://cdn.jsdelivr.net/gh/avivnofar/pdf-engine@js-dist/pdf_engine_js.js";
  await init();

  const pdf = new PdfEngine();
  const file = document.querySelector("input[type=file]").files[0];
  const { doc } = pdf.open(new Uint8Array(await file.arrayBuffer()), file.name);

  pdf.run("page_rotate", { doc, pages: [1], degrees: 90 });
  const { png } = pdf.renderPage(doc, 1, 96);   // Uint8Array PNG of page 1
  const bytes = pdf.save(doc);                   // Uint8Array, the edited PDF
</script>
```

| Method | What it does |
|---|---|
| `open(bytes, name?, password?)` | Open a PDF; returns its summary: `doc` (the id every other call takes), `pages`, `encrypted`… |
| `run(tool, args)` | Run any engine tool, return its JSON result |
| `call(tool, args)` | Run a tool, return every part: `[{ json }, { png, width, height }]` |
| `PdfEngine.tools()` | Every tool with its JSON Schema (189 of them: pages, comments, forms, redaction, bookmarks, security…) |
| `renderPage(doc, page, dpi?)` | Draw a page as PNG |
| `save(doc, full?, flattenFillSign?)` | The edited PDF as bytes (incremental by default) |
| `combine([{ name, bytes }, …])` | Merge PDFs into one |
| `extract(doc, pages)` | New PDF from chosen pages |
| `splitEvery(doc, n)` / `splitBefore(doc, pages)` | Split into several PDFs |
| `close(doc, discardChanges?)` | Close a document |

Pages are 1-based everywhere. Tools that take a file path (`doc_open`, `doc_save`, `doc_combine`,
`doc_split`) need a filesystem; in the browser use the methods above instead.

## Use it from Rust or the command line

The engine crates under `crates/` are ordinary Rust libraries, and `pdfcraft-cli` (in `apps/`)
drives the same tools from a shell or as an MCP server for AI agents. See
[FORK.md](FORK.md) for how the fork is laid out, how to build it, and how to pull in upstream
updates.

## License

MIT OR Apache-2.0, as upstream. Copyright (c) 2026 ArtCraft Team and the PdfCraft contributors;
fork additions copyright (c) 2026 the pdf-engine contributors. Notices are in [NOTICE](NOTICE),
asset credits in [ATTRIBUTION.md](ATTRIBUTION.md).
