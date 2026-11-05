/* tslint:disable */
/* eslint-disable */

/**
 * A PDF workbench with no UI. Open documents by bytes, edit them with tools, save them back to
 * bytes. Pages are 1-based.
 */
export class PdfEngine {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Run a tool and return every part of its result: `[{ json }, { png, width, height }]`.
     */
    call(tool: string, args: any): Array<any>;
    /**
     * Close a document; unsaved edits are refused unless `discardChanges` is true.
     */
    close(doc: number, discard_changes?: boolean | null): void;
    /**
     * Combine PDFs in order: `files` is an array of `{ name, bytes: Uint8Array }`.
     */
    combine(files: Array<any>): Uint8Array;
    /**
     * A new PDF with copies of these pages (1-based, in the order given).
     */
    extract(doc: number, pages: Uint32Array): Uint8Array;
    constructor();
    /**
     * Open a PDF. Returns its summary; `summary.doc` is the id other calls take.
     */
    open(bytes: Uint8Array, name?: string | null, password?: string | null): any;
    /**
     * Draw a page as PNG: `{ png: Uint8Array, width, height }`.
     */
    renderPage(doc: number, page: number, dpi?: number | null): any;
    /**
     * Run a tool and return its JSON result (`null` if it has none).
     */
    run(tool: string, args: any): any;
    /**
     * The document's bytes with every edit applied.
     */
    save(doc: number, full?: boolean | null, flatten_fill_sign?: boolean | null): Uint8Array;
    /**
     * Split before each of these pages (1-based): `[{ firstPage, lastPage, bytes }]`.
     */
    splitBefore(doc: number, pages: Uint32Array): Array<any>;
    /**
     * Split every `n` pages: `[{ firstPage, lastPage, bytes }]`.
     */
    splitEvery(doc: number, n: number): Array<any>;
    /**
     * Every tool with its JSON Schema.
     */
    static tools(): any;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_pdfengine_free: (a: number, b: number) => void;
    readonly pdfengine_call: (a: number, b: number, c: number, d: any) => [number, number, number];
    readonly pdfengine_close: (a: number, b: number, c: number) => [number, number];
    readonly pdfengine_combine: (a: number, b: any) => [number, number, number];
    readonly pdfengine_extract: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly pdfengine_new: () => number;
    readonly pdfengine_open: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly pdfengine_renderPage: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly pdfengine_run: (a: number, b: number, c: number, d: any) => [number, number, number];
    readonly pdfengine_save: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly pdfengine_splitBefore: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly pdfengine_splitEvery: (a: number, b: number, c: number) => [number, number, number];
    readonly pdfengine_tools: () => [number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
