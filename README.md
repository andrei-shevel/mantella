# Mantella

A fast, minimal desktop PDF reader built with **Tauri 2**, **Svelte 5**, and **pdfium** (Chromium's PDF engine) for native-quality rendering.

## Features

- **Library folder** — pick a folder of PDFs once; it's remembered and scanned recursively.
- **Live sync** — files copied into (or removed from) the folder appear/disappear in the sidebar automatically.
- **Search & pins** — explorer-style folder tree, filename search with flat results, pin favorites to the top for quick access.
- **Reading position** — the current page, scroll offset, and zoom are saved per file and restored when you reopen it (or relaunch the app).
- **Crisp rendering** — pages are rasterized by pdfium at the exact zoom × display scale, served through a custom `mantella://` protocol so they load as cached images.

### Shortcuts

| Keys | Action |
|---|---|
| ⌘F | Focus file search |
| ←/→, PgUp/PgDn | Previous / next page |
| ⌘+ / ⌘− | Zoom in / out |
| ⌘0 | Fit to width |
| ⌘/Ctrl + scroll | Zoom |

## Development

Prerequisites: Node 20+, Rust stable, and the [Tauri platform prerequisites](https://tauri.app/start/prerequisites/).

```sh
npm install
npm run setup        # downloads the pdfium dynamic library for your platform
npm run tauri dev
```

`npm run setup` fetches a prebuilt pdfium from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) into `src-tauri/resources/pdfium/` (gitignored, bundled into the app at build time). Pass `--target mac-arm64|mac-x64|win-x64|linux-x64|linux-arm64` to fetch for another platform.

### Checks

```sh
npm run check                 # svelte-check (frontend types)
cargo clippy                  # in src-tauri
cargo test                    # in src-tauri; exercises pdfium open + render
```

### Build

```sh
npm run tauri build
```

## Architecture

```
src/                      Svelte 5 frontend (runes)
  lib/api/                typed invoke/event wrappers — the only IPC surface
  lib/stores/             settings / library / reader state
  lib/components/         onboarding, library sidebar, reader (virtualized viewer)
src-tauri/src/
  pdf/engine.rs           pdfium worker thread (pdfium types are !Send, so all
                          PDF work is serialized onto one owning thread)
  pdf/renderer.rs         page → PNG at a requested pixel width
  pdf/protocol.rs         mantella://{docId}/{page}?w={px} scheme handler
  library/scanner.rs      recursive *.pdf scan
  library/watcher.rs      debounced fs watcher → `library-changed` event
  store/                  settings.json + files.json (per-file position/zoom/pin)
  commands/               #[tauri::command] handlers
```

Per-file reading state is keyed by absolute path in `files.json` in the app data directory; state of files deleted from the library is pruned automatically by the watcher.
