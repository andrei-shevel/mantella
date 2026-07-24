# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Mantella is a desktop PDF reader built with Tauri 2 (Rust backend) + Svelte 5 (frontend, runes) + pdfium (Chromium's PDF engine) for rendering.

## Commands

```sh
npm install
npm run setup            # one-time: downloads the pdfium dynamic library into src-tauri/resources/pdfium/ (gitignored)
npm run tauri dev        # run the app in dev mode
npm run tauri build      # production build

npm run check            # svelte-check (frontend type checking)
cd src-tauri && cargo clippy    # Rust lints
cd src-tauri && cargo test      # Rust tests (require pdfium — run `npm run setup` first)
```

Rust tests live in inline `#[cfg(test)]` modules (currently in `src-tauri/src/pdf/engine.rs`) and use PDF fixtures from `src-tauri/tests/fixtures/`. Run a single test with `cargo test <name>`.

## Architecture

Frontend and backend communicate only through two narrow surfaces:

1. **Typed IPC wrappers in `src/lib/api/`** — every `#[tauri::command]` (registered in `src-tauri/src/lib.rs`) has a typed wrapper in `commands.ts`, with request/response types in `types.ts` and Tauri events in `events.ts`. When adding a command, touch all three places: the Rust handler in `src-tauri/src/commands/`, the `generate_handler!` list in `lib.rs`, and the wrapper in `commands.ts`. Components never call `invoke` directly.

2. **The `mantella://` custom URI scheme (`src-tauri/src/pdf/protocol.rs`)** — page images are fetched as `mantella://{docId}/{page}?w={px}` and rendered by pdfium to PNG at the exact requested pixel width, so the webview treats them as cached images. Rendering does not go through `invoke`.

### The pdfium worker thread (key constraint)

pdfium-render types wrap raw `FPDF_*` pointers and are `!Send`, so **all** PDF work (open, render, text extraction, links) is serialized onto one dedicated worker thread owned by `PdfWorker` (`src-tauri/src/pdf/engine.rs`). The rest of the app holds a cheap cloneable handle and communicates via mpsc requests with oneshot replies. Never try to move pdfium types across threads; add new PDF operations as new `Request` variants handled on the worker.

### Backend layout (`src-tauri/src/`)

- `pdf/` — engine (worker thread), renderer (page → PNG), text/links extraction, `mantella://` protocol handler
- `library/` — recursive `*.pdf` scanner, a debounced fs watcher that emits `library-changed` events and prunes state of deleted files, and `identity.rs` (content-based file id: partial hash + size, so per-file state survives renames/moves)
- `store/` — persisted JSON in the app data dir: `settings.json` (library path, last file, shortcut overrides, theme) and `files.json` (per-file page/scroll/zoom/pin/bookmarks, keyed by content id from `library/identity.rs`, not path)
- `commands/` — thin `#[tauri::command]` handlers grouped by domain (library, pdf, reading, bookmarks)
- `state.rs` — `AppState` (store + PdfWorker + watcher) managed by Tauri

macOS "Open With"/double-click file opens arrive as Apple events (not argv); they are buffered in `PendingOpenFiles` and drained by the frontend via `take_pending_open_files` after an `open-file` event (see the `RunEvent::Opened` handler in `lib.rs`).

### Frontend layout (`src/lib/`)

- `stores/` — Svelte 5 rune-based class singletons (`settings`, `library`, `reader`, `ui`, `shortcuts`). `reader.svelte.ts` owns document lifecycle: it guards against stale async opens (user switching files mid-load), debounces reading-position saves, and hands the restore position to the Viewer via `pendingRestore`.
- `components/` — `onboarding/` (welcome/folder pick), `library/` (sidebar, folder tree, search, pins), `reader/` (virtualized `Viewer.svelte`, `Page.svelte`, toolbar, selectable text layer in `textLayer.ts`, bookmarks panel/markers), `settings/` (settings modal, update prompt), `common/` (shared UI: context menu, empty state, icon)
- `shortcuts.ts` — rebindable shortcut definitions + defaults, used by the `shortcuts` store

Keyboard navigation in the sidebar file list and bookmarks panel uses a roving-cursor pattern (single container `tabindex`, `aria-activedescendant` pointing at the active row) rather than per-row `tabindex`, so arrow keys move a `cursor`/`bookmarkCursor` key tracked on `library`/`reader` instead of the DOM focus. Each store also exposes its container's element (`listEl`, `viewerEl`, `bookmarksListEl`) so other stores can move focus across panels — e.g. ⌘← focuses the sidebar if closed, or hands focus to the viewer if the sidebar was already focused when it closes.

Per-file reading state (page, scroll offset, zoom, pinned, bookmarks) is restored on open and saved debounced; zoom `null` means fit-to-width. Identity is content-based (see `library/identity.rs` above), so this state survives the file being renamed or moved.

The theme setting (`system` | `light` | `dark`) is resolved to a concrete `light`/`dark` in `stores/settings.svelte.ts` (via `matchMedia`, not a CSS media query), which sets `data-theme` on `<html>`. `styles/app.css` only ever branches on `:root[data-theme="dark"]`/`[data-theme="light"]`, so an explicit choice can override the OS preference; `system` re-resolves live via a `matchMedia` change listener.
