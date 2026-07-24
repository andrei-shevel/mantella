# Mantella

A fast, minimal desktop PDF reader built with **Tauri 2**, **Svelte 5**, and **pdfium** (Chromium's PDF engine) for native-quality rendering.

**[mantella.bearrr.io](https://mantella.bearrr.io/)**

## Features

- **Library folder** — pick a folder of PDFs once; it's remembered and scanned recursively.
- **Live sync** — files copied into (or removed from) the folder appear/disappear in the sidebar automatically.
- **Search & pins** — explorer-style folder tree, filename search with flat results, pin favorites to the top for quick access.
- **Reading position** — the current page, scroll offset, and zoom are saved per file (keyed by content, so it survives the file being renamed or moved) and restored when you reopen it.
- **Bookmarks** — save named positions within a document and jump back to them from the bookmarks panel.
- **Keyboard-first navigation** — arrow keys move through the file list and bookmarks panel, with focus handed off cleanly between the sidebar, bookmarks panel, and document.
- **Context menu actions** — right-click a file or folder to reveal it in Finder/Explorer.
- **Customizable shortcuts** — rebind navigation, zoom, and panel shortcuts from Settings.
- **Crisp rendering** — pages are rasterized by pdfium at the exact zoom × display scale, served through a custom `mantella://` protocol so they load as cached images.
- **Auto-updates** — signed releases from GitHub; the app checks for updates on launch and from Settings.

### Shortcuts

Defaults — rebind any of these from Settings (⌘,):

| Keys    | Action                 |
| ------- | ---------------------- |
| ⌘↑ / ⌘↓ | Previous / next page   |
| ⌘+ / ⌘− | Zoom in / out          |
| ⌘0      | Fit to width           |
| ⌘←      | Toggle files panel     |
| ⌘→      | Toggle bookmarks panel |
| ⌘F      | Focus file search      |

Fixed:

| Keys            | Action                 |
| --------------- | ---------------------- |
| PgUp/PgDn       | Previous / next page   |
| Home / End      | First / last page      |
| ⌘/Ctrl + scroll | Zoom                   |
| ⌘1 – ⌘9         | Open pinned file       |
| ⌘O              | Open PDF…              |
| ⌘⇧O             | Change library folder… |
| ⌘,              | Settings…              |

When the file list or bookmarks panel is focused: ↑ / ↓ moves the selection, Enter/Space opens it, ← / → collapses/expands a folder, Home/End jump to the first/last row, and Esc returns focus to the document.

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
npm run format:check          # prettier
cargo clippy                  # in src-tauri
cargo test                    # in src-tauri; exercises pdfium open + render
```

### Build

```sh
# Signing key required when createUpdaterArtifacts is enabled (see Releases).
export TAURI_SIGNING_PRIVATE_KEY_PATH="$PWD/.tauri/mantella.key"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
npm run tauri build
```

## Releases & auto-updates

Mantella ships updates via [GitHub Releases](https://github.com/andrei-shevel/mantella/releases). The app checks `latest.json` on that release and installs signed updater artifacts. macOS builds from CI are **Developer ID signed and notarized** so Gatekeeper accepts downloads without the “damaged and can’t be opened” quarantine error.

### One-time setup

#### Updater signing key

1. Keep the private key in `.tauri/mantella.key` (gitignored). The matching public key is already in `src-tauri/tauri.conf.json`.
2. Add GitHub Actions secrets on the repo:
   - `TAURI_SIGNING_PRIVATE_KEY` — full contents of `.tauri/mantella.key`
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — empty string if the key has no password
3. Ensure Actions have read/write contents permission (Settings → Actions → General → Workflow permissions).

```sh
gh secret set TAURI_SIGNING_PRIVATE_KEY < .tauri/mantella.key
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --body ""
```

If you lose the private key, already-installed apps cannot verify new updates — generate a new keypair only when you accept that break.

#### macOS Developer ID + notarization

Requires an Apple Developer Program membership.

1. Create a **Developer ID Application** certificate (Certificates, Identifiers & Profiles → Certificates). Export it from Keychain Access as a `.p12` with a password.
2. Base64-encode the `.p12`:

```sh
openssl base64 -A -in /path/to/certificate.p12 -out certificate-base64.txt
```

3. Create an App Store Connect API key (Users and Access → Integrations → Team Keys) with at least Developer access. Note the Issuer ID and Key ID; download the `.p8` (only once).
4. Add these GitHub Actions secrets:

| Secret                       | Value                                 |
| ---------------------------- | ------------------------------------- |
| `APPLE_CERTIFICATE`          | Contents of `certificate-base64.txt`  |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the `.p12`               |
| `KEYCHAIN_PASSWORD`          | Any password for the CI temp keychain |
| `APPLE_API_ISSUER`           | Issuer ID from App Store Connect      |
| `APPLE_API_KEY`              | Key ID from App Store Connect         |
| `APPLE_API_KEY_CONTENT`      | Full contents of the `.p8` file       |

```sh
gh secret set APPLE_CERTIFICATE < certificate-base64.txt
gh secret set APPLE_CERTIFICATE_PASSWORD
gh secret set KEYCHAIN_PASSWORD
gh secret set APPLE_API_ISSUER
gh secret set APPLE_API_KEY
gh secret set APPLE_API_KEY_CONTENT < /path/to/AuthKey_XXXXX.p8
```

Local macOS builds can also sign/notarize if the cert is in your login keychain and you export `APPLE_SIGNING_IDENTITY`, `APPLE_API_ISSUER`, `APPLE_API_KEY`, and `APPLE_API_KEY_PATH`. CI remains the source of truth for Release artifacts.

### Publish a release

1. Bump `version` in `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` to the same value.
2. Commit, then tag and push:

```sh
git tag v0.2.0
git push origin v0.2.0
```

3. The [`release`](.github/workflows/release.yml) workflow builds macOS (arm64 + x64, signed + notarized), Windows, and Linux, creates a draft GitHub Release with installers + updater signatures, and uploads `latest.json` for the in-app updater.
4. Review the draft release and publish it.

Users on an older build are prompted on launch (and can check from Settings → Updates).

### Verify a macOS build

After downloading the DMG from a draft Release onto a clean Mac:

```sh
spctl --assess -vv --type install /path/to/Mantella.app
xcrun stapler validate /path/to/Mantella.dmg
codesign -dv --verbose=4 /path/to/Mantella.app
codesign -dv --verbose=4 /path/to/Mantella.app/Contents/Frameworks/libpdfium.dylib
```

The app should open without a Gatekeeper “damaged” dialog. `libpdfium.dylib` must share the same Team ID as the app.

## Architecture

```
src/                      Svelte 5 frontend (runes)
  lib/api/                typed invoke/event wrappers — the only IPC surface
  lib/stores/             settings / library / reader / shortcuts state
  lib/shortcuts.ts        rebindable shortcut definitions + defaults
  lib/components/         onboarding, library sidebar, reader (virtualized
                          viewer, bookmarks panel), settings, shared common
src-tauri/src/
  pdf/engine.rs           pdfium worker thread (pdfium types are !Send, so all
                          PDF work is serialized onto one owning thread)
  pdf/renderer.rs         page → PNG at a requested pixel width
  pdf/protocol.rs         mantella://{docId}/{page}?w={px} scheme handler
  library/scanner.rs      recursive *.pdf scan
  library/watcher.rs      debounced fs watcher → `library-changed` event
  library/identity.rs     content-based file id (partial hash + size) so
                          per-file state survives renames/moves
  store/                  settings.json + files.json (per-file position/zoom/
                          pin/bookmarks, keyed by content id)
  commands/               #[tauri::command] handlers (library, pdf, reading,
                          bookmarks)
```

Per-file reading state is keyed by a content-based id (partial hash + size, see `library/identity.rs`) in `files.json` in the app data directory, so it survives the file being renamed or moved; state of files deleted from the library is pruned automatically by the watcher.

## License

[MIT](LICENSE)
